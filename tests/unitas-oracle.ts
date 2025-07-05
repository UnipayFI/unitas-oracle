import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { UnitasOracle } from "../target/types/unitas_oracle";
import {
  initAdminConfig,
  getConfigAddress,
  proposeNewAdmin,
  acceptAdminTransfer,
  addOperator,
  removeOperator,
  getOperatorAddress,
  createAssetLookupTable,
  getAssetLookupTableAddress,
  addAccount,
  removeAccount,
  updateAumUsd,
} from "../helpers/instructions";
import { assert } from "chai";

describe("unitas-oracle", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.UnitasOracle as Program<UnitasOracle>;
  const provider = anchor.getProvider();
  const admin = anchor.web3.Keypair.generate();
  const operator = anchor.web3.Keypair.generate();

  let currentAdmin = admin;

  before(async () => {
    // Airdrop to the admin
    let airdropTx = await provider.connection.requestAirdrop(
      admin.publicKey,
      100 * anchor.web3.LAMPORTS_PER_SOL
    );
    let latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      signature: airdropTx,
    });

    // Airdrop to the operator
    airdropTx = await provider.connection.requestAirdrop(
      operator.publicKey,
      100 * anchor.web3.LAMPORTS_PER_SOL
    );
    latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      signature: airdropTx,
    });
  });

  it("Is initialized and adds first operator!", async () => {
    const tx = await initAdminConfig(program, currentAdmin);
    console.log("Your transaction signature", tx);

    const [configAddress] = getConfigAddress(program);
    const configAccount = await program.account.config.fetch(configAddress);
    assert.ok(configAccount.admin.equals(currentAdmin.publicKey));
    assert.ok(configAccount.pendingAdmin === null);

    // Add the main operator for tests
    await addOperator(program, currentAdmin, operator.publicKey);
    const [operatorAddress] = getOperatorAddress(program, operator.publicKey);
    const operatorAccount = await program.account.operator.fetch(operatorAddress);
    assert.ok(operatorAccount.user.equals(operator.publicKey));
  });

  it("Transfers admin", async () => {
    const newAdmin = anchor.web3.Keypair.generate();
    // Airdrop to the new admin
    const airdropTx = await provider.connection.requestAirdrop(
      newAdmin.publicKey,
      100 * anchor.web3.LAMPORTS_PER_SOL
    );
    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      signature: airdropTx,
    });

    // Propose new admin
    await proposeNewAdmin(program, currentAdmin, newAdmin.publicKey);

    const [configAddress] = getConfigAddress(program);
    let configAccount = await program.account.config.fetch(configAddress);
    assert.ok(configAccount.pendingAdmin.equals(newAdmin.publicKey));

    // Accept admin transfer
    await acceptAdminTransfer(program, newAdmin);

    configAccount = await program.account.config.fetch(configAddress);
    assert.ok(configAccount.admin.equals(newAdmin.publicKey));
    assert.ok(configAccount.pendingAdmin === null);

    // Update current admin for subsequent tests
    currentAdmin = newAdmin;
  });

  it("Manages operators", async () => {
    const operatorUser = anchor.web3.Keypair.generate();

    // Add operator
    await addOperator(program, currentAdmin, operatorUser.publicKey);

    const [operatorAddress] = getOperatorAddress(
      program,
      operatorUser.publicKey
    );
    const operatorAccount = await program.account.operator.fetch(
      operatorAddress
    );
    assert.ok(operatorAccount.user.equals(operatorUser.publicKey));

    // Remove operator
    await removeOperator(program, currentAdmin, operatorUser.publicKey);

    try {
      await program.account.operator.fetch(operatorAddress);
      assert.fail("Operator account should have been closed");
    } catch (e) {
      assert.include(e.message, "Account does not exist");
    }
  });

  it("Manages asset lookup table", async () => {
    const assetIndex = 0;
    const jlpAccount1 = anchor.web3.Keypair.generate().publicKey;
    const jlpAccount2 = anchor.web3.Keypair.generate().publicKey;

    // Create asset lookup table
    const mint = anchor.web3.Keypair.generate().publicKey;
    const decimals = 6;
    await createAssetLookupTable(
      program,
      currentAdmin,
      assetIndex,
      mint,
      decimals
    );
    const [assetLookupTableAddress] = getAssetLookupTableAddress(
      program,
      assetIndex
    );
    let assetLookupTableAccount = await program.account.assetLookupTable.fetch(
      assetLookupTableAddress
    );
    assert.strictEqual(assetLookupTableAccount.index, assetIndex);
    assert.lengthOf(assetLookupTableAccount.accounts, 0);

    // Add accounts using the operator
    await addAccount(program, operator, assetIndex, jlpAccount1);
    await addAccount(program, operator, assetIndex, jlpAccount2);
    assetLookupTableAccount = await program.account.assetLookupTable.fetch(
      assetLookupTableAddress
    );
    assert.lengthOf(assetLookupTableAccount.accounts, 2);
    assert.ok(
      assetLookupTableAccount.accounts.some((pubkey) =>
        pubkey.equals(jlpAccount1)
      )
    );
    assert.ok(
      assetLookupTableAccount.accounts.some((pubkey) =>
        pubkey.equals(jlpAccount2)
      )
    );

    // Remove account using the operator
    await removeAccount(program, operator, assetIndex, jlpAccount1);
    assetLookupTableAccount = await program.account.assetLookupTable.fetch(
      assetLookupTableAddress
    );
    assert.lengthOf(assetLookupTableAccount.accounts, 1);
    assert.isFalse(
      assetLookupTableAccount.accounts.some((pubkey) =>
        pubkey.equals(jlpAccount1)
      )
    );

    // Update AUM USD using the operator
    const aum = new anchor.BN(123456789);
    await updateAumUsd(program, operator, assetIndex, aum);
    assetLookupTableAccount = await program.account.assetLookupTable.fetch(
      assetLookupTableAddress
    );
    assert.ok(assetLookupTableAccount.aumUsd.eq(aum));

    // Add accounts using the operator
    const account1 = anchor.web3.Keypair.generate().publicKey;
    const account2 = anchor.web3.Keypair.generate().publicKey;
    await addAccount(program, operator, assetIndex, account1);
    await addAccount(program, operator, assetIndex, account2);
    assetLookupTableAccount = await program.account.assetLookupTable.fetch(
      assetLookupTableAddress
    );
    assert.lengthOf(assetLookupTableAccount.accounts, 3);
    assert.ok(
      assetLookupTableAccount.accounts.some((pubkey) =>
        pubkey.equals(account1)
      )
    );
    assert.ok(
      assetLookupTableAccount.accounts.some((pubkey) =>
        pubkey.equals(account2)
      )
    );

    // Remove account using the operator
    await removeAccount(program, operator, assetIndex, account1);
    assetLookupTableAccount = await program.account.assetLookupTable.fetch(
      assetLookupTableAddress
    );
    assert.lengthOf(assetLookupTableAccount.accounts, 2);
    assert.isFalse(
      assetLookupTableAccount.accounts.some((pubkey) =>
        pubkey.equals(account1)
      )
    );
  });
});
