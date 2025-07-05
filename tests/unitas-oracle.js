"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
const anchor = __importStar(require("@coral-xyz/anchor"));
const instructions_1 = require("../helpers/instructions");
const chai_1 = require("chai");
describe("unitas-oracle", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.UnitasOracle;
    const provider = anchor.getProvider();
    const admin = anchor.web3.Keypair.generate();
    const operator = anchor.web3.Keypair.generate();
    let currentAdmin = admin;
    before(() => __awaiter(void 0, void 0, void 0, function* () {
        // Airdrop to the admin
        let airdropTx = yield provider.connection.requestAirdrop(admin.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);
        let latestBlockhash = yield provider.connection.getLatestBlockhash();
        yield provider.connection.confirmTransaction({
            blockhash: latestBlockhash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
            signature: airdropTx,
        });
        // Airdrop to the operator
        airdropTx = yield provider.connection.requestAirdrop(operator.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);
        latestBlockhash = yield provider.connection.getLatestBlockhash();
        yield provider.connection.confirmTransaction({
            blockhash: latestBlockhash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
            signature: airdropTx,
        });
    }));
    it("Is initialized and adds first operator!", () => __awaiter(void 0, void 0, void 0, function* () {
        const tx = yield (0, instructions_1.initAdminConfig)(program, currentAdmin);
        console.log("Your transaction signature", tx);
        const [configAddress] = (0, instructions_1.getConfigAddress)(program);
        const configAccount = yield program.account.config.fetch(configAddress);
        chai_1.assert.ok(configAccount.admin.equals(currentAdmin.publicKey));
        chai_1.assert.ok(configAccount.pendingAdmin === null);
        // Add the main operator for tests
        yield (0, instructions_1.addOperator)(program, currentAdmin, operator.publicKey);
        const [operatorAddress] = (0, instructions_1.getOperatorAddress)(program, operator.publicKey);
        const operatorAccount = yield program.account.operator.fetch(operatorAddress);
        chai_1.assert.ok(operatorAccount.user.equals(operator.publicKey));
    }));
    it("Transfers admin", () => __awaiter(void 0, void 0, void 0, function* () {
        const newAdmin = anchor.web3.Keypair.generate();
        // Airdrop to the new admin
        const airdropTx = yield provider.connection.requestAirdrop(newAdmin.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);
        const latestBlockhash = yield provider.connection.getLatestBlockhash();
        yield provider.connection.confirmTransaction({
            blockhash: latestBlockhash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
            signature: airdropTx,
        });
        // Propose new admin
        yield (0, instructions_1.proposeNewAdmin)(program, currentAdmin, newAdmin.publicKey);
        const [configAddress] = (0, instructions_1.getConfigAddress)(program);
        let configAccount = yield program.account.config.fetch(configAddress);
        chai_1.assert.ok(configAccount.pendingAdmin.equals(newAdmin.publicKey));
        // Accept admin transfer
        yield (0, instructions_1.acceptAdminTransfer)(program, newAdmin);
        configAccount = yield program.account.config.fetch(configAddress);
        chai_1.assert.ok(configAccount.admin.equals(newAdmin.publicKey));
        chai_1.assert.ok(configAccount.pendingAdmin === null);
        // Update current admin for subsequent tests
        currentAdmin = newAdmin;
    }));
    it("Manages operators", () => __awaiter(void 0, void 0, void 0, function* () {
        const operatorUser = anchor.web3.Keypair.generate();
        // Add operator
        yield (0, instructions_1.addOperator)(program, currentAdmin, operatorUser.publicKey);
        const [operatorAddress] = (0, instructions_1.getOperatorAddress)(program, operatorUser.publicKey);
        const operatorAccount = yield program.account.operator.fetch(operatorAddress);
        chai_1.assert.ok(operatorAccount.user.equals(operatorUser.publicKey));
        // Remove operator
        yield (0, instructions_1.removeOperator)(program, currentAdmin, operatorUser.publicKey);
        try {
            yield program.account.operator.fetch(operatorAddress);
            chai_1.assert.fail("Operator account should have been closed");
        }
        catch (e) {
            chai_1.assert.include(e.message, "Account does not exist");
        }
    }));
    it("Manages asset lookup table", () => __awaiter(void 0, void 0, void 0, function* () {
        const assetIndex = 0;
        const jlpAccount1 = anchor.web3.Keypair.generate().publicKey;
        const jlpAccount2 = anchor.web3.Keypair.generate().publicKey;
        // Create asset lookup table
        yield (0, instructions_1.createAssetLookupTable)(program, currentAdmin, assetIndex);
        const [assetLookupTableAddress] = (0, instructions_1.getAssetLookupTableAddress)(program, assetIndex);
        let assetLookupTableAccount = yield program.account.assetLookupTable.fetch(assetLookupTableAddress);
        chai_1.assert.strictEqual(assetLookupTableAccount.index, assetIndex);
        chai_1.assert.lengthOf(assetLookupTableAccount.jlpAccounts, 0);
        // Add JLP accounts using the operator
        yield (0, instructions_1.addJlpAccount)(program, operator, assetIndex, jlpAccount1);
        yield (0, instructions_1.addJlpAccount)(program, operator, assetIndex, jlpAccount2);
        assetLookupTableAccount = yield program.account.assetLookupTable.fetch(assetLookupTableAddress);
        chai_1.assert.lengthOf(assetLookupTableAccount.jlpAccounts, 2);
        chai_1.assert.ok(assetLookupTableAccount.jlpAccounts.some((pubkey) => pubkey.equals(jlpAccount1)));
        chai_1.assert.ok(assetLookupTableAccount.jlpAccounts.some((pubkey) => pubkey.equals(jlpAccount2)));
        // Remove a JLP account using the operator
        yield (0, instructions_1.removeJlpAccount)(program, operator, assetIndex, jlpAccount1);
        assetLookupTableAccount = yield program.account.assetLookupTable.fetch(assetLookupTableAddress);
        chai_1.assert.lengthOf(assetLookupTableAccount.jlpAccounts, 1);
        chai_1.assert.isFalse(assetLookupTableAccount.jlpAccounts.some((pubkey) => pubkey.equals(jlpAccount1)));
        // Update AUM USD using the operator
        const aum = new anchor.BN(123456789);
        yield (0, instructions_1.updateAumUsd)(program, operator, assetIndex, aum);
        assetLookupTableAccount = yield program.account.assetLookupTable.fetch(assetLookupTableAddress);
        chai_1.assert.ok(assetLookupTableAccount.aumUsd.eq(aum));
    }));
});
