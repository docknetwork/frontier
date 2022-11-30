import { expect } from "chai";
import * as $ from "parity-scale-codec";

import PalletStorageAccessor from "../build/contracts/PalletStorageAccessor.json";
import { createAndFinalizeBlock, customRequest, describeWithFrontier } from "./util";
import { AbiItem } from "web3-utils";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./config";

describeWithFrontier("Frontier RPC (Storage accessor)", (context) => {
	const TEST_CONTRACT_BYTECODE = PalletStorageAccessor.bytecode;
	const TEST_CONTRACT_ABI = PalletStorageAccessor.abi as AbiItem[];
	let contractAddress;

	before("create the contract", async function () {
		this.timeout(15000);
		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: TEST_CONTRACT_BYTECODE,
				value: "0x00",
				gasPrice: "0x3B9ACA00",
				gas: "0x200000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);
		const resp = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
		await createAndFinalizeBlock(context.web3);
		const receipt = await context.web3.eth.getTransactionReceipt(resp.result);
		contractAddress = receipt.contractAddress;
	});

	it("get transaction by hash", async () => {
		const latestBlock = await context.web3.eth.getBlock("latest");
		expect(latestBlock.transactions.length).to.equal(1);

		const txHash = latestBlock.transactions[0];
		const tx = await context.web3.eth.getTransaction(txHash);
		expect(tx.hash).to.equal(txHash);
	});

	it("should fail on invalid pallet", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods.getStorage("A", "Number", 1, contractAddress, "0x").call();
		expect(success).to.equal(false);
		expect(rawData).to.equal(null);
	});

	it("should fail on invalid key", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods.getStorage("EVM", "AccountCodes", 0, "0x", "0x").call();
		expect(success).to.equal(false);
		expect(rawData).to.equal(null);
	});

	it("should fail on invalid storage member", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods.getStorage("EVM", "ABCDE", 1, contractAddress, "0x").call();
		expect(success).to.equal(false);
		expect(rawData).to.equal(null);
	});

	it("should read a value with no key", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods.getStorage("System", "Number", 0, "0x", "0x").call();
		expect(success).to.equal(true);
		const buffer = Buffer.from(rawData.slice(2), "hex");
		const [found, ...data] = Array.from(buffer);
		const blockNumber = $.u32.decode(new Uint8Array(data)).toFixed();
		expect(!!found).to.equal(true);
		expect(blockNumber).to.equal("1");
	});

	it("should read a value with no key", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods.getStorage("Timestamp", "Now", 0, "0x", "0x").call();
		expect(success).to.equal(true);
		const buffer = Buffer.from(rawData.slice(2), "hex");
		const [found, ...data] = Array.from(buffer);
		const timestamp = $.u64.decode(new Uint8Array(data)).toString();
		expect(!!found).to.equal(true);
		expect(timestamp).to.equal("6000");
	});

	it("should read value from a single key map", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods
			.getStorage("EVM", "AccountCodes", 1, contractAddress, "0x")
			.call();
		const buffer = Buffer.from(rawData.slice(2), "hex");
		expect(success).to.equal(true);
		const [found, ...data] = Array.from(buffer);
		const code = $.array($.u8).decode(new Uint8Array(data));
		expect(!!found).to.equal(true);
		expect(code).to.deep.equal(Array.from(Buffer.from(PalletStorageAccessor.deployedBytecode.slice(2), "hex")));
	});

	it("should read value with offset", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods
			.getStorageWithOffset("EVM", "AccountCodes", 1, contractAddress, "0x", 100)
			.call();
		expect(success).to.equal(true);
		const buffer = Buffer.from(rawData.slice(2), "hex");
		const [found, ...data] = Array.from(buffer);
		const code = Array.from(data);
		expect(!!found).to.equal(true);
		expect(code).to.deep.equal(
			Array.from(Buffer.from(PalletStorageAccessor.deployedBytecode.slice(2), "hex")).slice(98)
		);
	});

	it("should read value with len", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		const [success, rawData] = await contract.methods
			.getStorageWithLen("EVM", "AccountCodes", 1, contractAddress, "0x", 4)
			.call();
		expect(success).to.equal(true);
		const buffer = Buffer.from(rawData.slice(2), "hex");
		const [found, ...data] = Array.from(buffer);
		const len = $.compact.decode(new Uint8Array(data));
		expect(!!found).to.equal(true);
		expect(len).to.equal(Buffer.from(PalletStorageAccessor.deployedBytecode.slice(2), "hex").length);
	});

	it("should read value with offset and len", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		let [success, rawData] = await contract.methods
			.getStorageWithOffsetLen("EVM", "AccountCodes", 1, contractAddress, "0x", 2, 1000)
			.call();
		expect(success).to.equal(true);
		const buffer = Buffer.from(rawData.slice(2), "hex");
		const [found, ...data] = Array.from(buffer);
		const code = Array.from(data);
		expect(!!found).to.equal(true);
		expect(code).to.deep.equal(
			Array.from(Buffer.from(PalletStorageAccessor.deployedBytecode.slice(2), "hex")).slice(0, 1000)
		);
	});

	it("should check value from a single key map for existence with offset and 0 len", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		let [success, rawData] = await contract.methods
			.getStorageWithOffsetLen("EVM", "AccountCodes", 1, contractAddress, "0x", 2, 0)
			.call();
		expect(success).to.equal(true);
		const [found] = Array.from(Buffer.from(rawData.slice(2), "hex"));
		expect(!!found).to.equal(true);
	});

	it("should read value with offset and len", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});

		let [success, rawData] = await contract.methods
			.getStorageWithOffsetLen("EVM", "AccountCodes", 1, contractAddress, "0x", 20000, 10)
			.call();
		expect(success).to.equal(true);
		const [found] = Array.from(Buffer.from(rawData.slice(2), "hex"));
		expect(!!found).to.equal(true);
	});

	it("should read value from a double key map", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});
		let [success, rawData] = await contract.methods
			.getStorage(
				"EVM",
				"AccountStorages",
				2,
				"0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
				$.u256.encode(BigInt(123)).reverse()
			)
			.call();
		const [found, ...data] = Array.from(Buffer.from(rawData.slice(2), "hex"));
		expect(success).to.equal(true);
		expect(!!found).to.equal(true);
		expect($.u256.decode(new Uint8Array(data.reverse()))).to.deep.equal(BigInt(456));
	});

	it("should read default value from a double key map", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});
		let [success, rawData] = await contract.methods
			.getStorage(
				"EVM",
				"AccountStorages",
				2,
				"0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
				$.u256.encode(BigInt(0)).reverse()
			)
			.call();
		expect(success).to.equal(true);
		const [found, ...data] = Array.from(Buffer.from(rawData.slice(2), "hex"));
		expect(!!found).to.equal(true);
		expect($.u256.decode(new Uint8Array(data))).to.deep.equal(BigInt(0));
	});

	it("should read default with offset", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});
		let [success, rawData] = await contract.methods
			.getStorageWithOffset(
				"EVM",
				"AccountStorages",
				2,
				"0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
				$.u256.encode(BigInt(0)).reverse(),
				10
			)
			.call();
		expect(success).to.equal(true);
		const [found, ...data] = Array.from(Buffer.from(rawData.slice(2), "hex"));
		expect(!!found).to.equal(true);
		expect(data).to.deep.equal(new Array(22).fill(0));
	});

	it("should read default with len", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});
		let [success, rawData] = await contract.methods
			.getStorageWithLen(
				"EVM",
				"AccountStorages",
				2,
				"0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
				$.u256.encode(BigInt(0)).reverse(),
				5
			)
			.call();
		expect(success).to.equal(true);
		const [found, ...data] = Array.from(Buffer.from(rawData.slice(2), "hex"));
		expect(!!found).to.equal(true);
		expect(data).to.deep.equal(new Array(5).fill(0));
	});

	it("should read default with offset and len", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});
		let [success, rawData] = await contract.methods
			.getStorageWithOffsetLen(
				"EVM",
				"AccountStorages",
				2,
				"0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
				$.u256.encode(BigInt(0)).reverse(),
				30,
				5
			)
			.call();
		const [found, ...data] = Array.from(Buffer.from(rawData.slice(2), "hex"));
		expect(!!found).to.equal(true);
		expect(data).to.deep.equal(new Array(2).fill(0));
	});
});
