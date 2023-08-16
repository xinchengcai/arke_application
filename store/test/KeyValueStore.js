// ./test/KeyValueStore.js
const KeyValueStore = artifacts.require("KeyValueStore");

contract("KeyValueStore", () => {
  it("...should deploy and successfully call createInstance using the method's provided gas estimate", async () => {
    const KeyValueStoreInstance = await KeyValueStore.new();

    // random test parameters
    const cipher = "0xabcdef123456";
    const iv = "0xabcdef123456"
    const addr = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    const id = "some_id";

    const gasEstimateWrite = await KeyValueStoreInstance.Write.estimateGas(cipher, iv, addr, id);
    console.log(`Gas estimate for Write transaction: ${gasEstimateWrite}`);
    const tx0 = await KeyValueStoreInstance.Write(cipher, iv, addr, id,{
      gas: gasEstimateWrite
    });
    assert(tx0);
    const gasEstimateRead = await KeyValueStoreInstance.Read.estimateGas(addr);
    console.log(`Gas estimate for Read transaction: ${gasEstimateRead}`);
    const tx1 = await KeyValueStoreInstance.Read(addr,{
      gas: gasEstimateRead
    });
    assert(tx1);
    const gasEstimateDelete = await KeyValueStoreInstance.Delete.estimateGas(addr);
    console.log(`Gas estimate for Delete transaction: ${gasEstimateDelete}`);
    const tx2 = await KeyValueStoreInstance.Delete(addr,{
      gas: gasEstimateDelete
    });
    assert(tx2);
    const gasEstimateSendEther = await KeyValueStoreInstance.sendEther.estimateGas(addr);
    console.log(`Gas estimate for sendEther transaction: ${gasEstimateSendEther}`);
    const tx3 = await KeyValueStoreInstance.sendEther(addr,{
      gas: gasEstimateSendEther
    });
    assert(tx3);

    const gasPrice = await web3.eth.getGasPrice();
    const gasCostInWeiWrite = BigInt(gasEstimateWrite) * BigInt(gasPrice);
    const gasCostInEtherWrite = web3.utils.fromWei(gasCostInWeiWrite.toString(), 'ether');
    console.log(`Cost of the Write transaction: ${gasCostInEtherWrite} Eth`);
    const gasCostInWeiRead = BigInt(gasEstimateRead) * BigInt(gasPrice);
    const gasCostInEtherRead = web3.utils.fromWei(gasCostInWeiRead.toString(), 'ether');
    console.log(`Cost of the Read transaction: ${gasCostInEtherRead} Eth`);
    const gasCostInWeiDelete = BigInt(gasEstimateDelete) * BigInt(gasPrice);
    const gasCostInEtherDelete = web3.utils.fromWei(gasCostInWeiDelete.toString(), 'ether');
    console.log(`Cost of the Delete transaction: ${gasCostInEtherDelete} Eth`);
    const gasCostInWeiSendEther = BigInt(gasEstimateSendEther) * BigInt(gasPrice);
    const gasCostInEtherSendEther = web3.utils.fromWei(gasCostInWeiSendEther.toString(), 'ether');
    console.log(`Cost of the sendEther transaction: ${gasCostInEtherSendEther} Eth`);
  });
});