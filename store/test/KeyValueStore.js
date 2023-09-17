/*
// ---------------------------------------
// File: KeyValueStore.js
// Date: 01 Sept 2023
// Description: Performance Evaluation 
//              (Transaction gas estimation)
// ---------------------------------------
const KeyValueStore = artifacts.require("KeyValueStore");

contract("KeyValueStore", () => {
  it("...should deploy and successfully call createInstance using the method's provided gas estimate", async () => {
    const KeyValueStoreInstance = await KeyValueStore.new();
    const currentGasPrice = await web3.eth.getGasPrice();
    const networkId = await web3.eth.net.getId();
    console.log(`Connected to network ID ${networkId}, Gas price: ${currentGasPrice} wei`);

    // Random test parameters
    //const cipher = new Uint8Array(1000).fill(0x01); // 1k-byte ciphertext
    const cipher = "0x01"; // one-byte ciphertext
    const iv = "0x1234567890abcdef12345678";      
    const addr = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    const id = ["12345678"];

    // Write
    const gasEstimateWrite = await KeyValueStoreInstance.Write.estimateGas(cipher, iv, addr, id);
    console.log(`Gas estimate for Write transaction: ${gasEstimateWrite}`);
    const gasCostInWeiWrite = BigInt(gasEstimateWrite) * BigInt(currentGasPrice);
    const gasCostInEtherWrite = web3.utils.fromWei(gasCostInWeiWrite.toString(), 'ether');
    console.log(`\nCost of the Write transaction: ${gasCostInEtherWrite* 1_000_000_000.0} gwei`);
    const tx0 = await KeyValueStoreInstance.Write(cipher, iv, addr, id,{
      gas: gasEstimateWrite,
      gasPrice: currentGasPrice 
    });
    assert(tx0);

    // Read
    const gasEstimateRead = await KeyValueStoreInstance.Read.estimateGas(addr);
    console.log(`\nGas estimate for Read transaction: ${gasEstimateRead}`);
    const gasCostInWeiRead = BigInt(gasEstimateRead) * BigInt(currentGasPrice);
    const gasCostInEtherRead = web3.utils.fromWei(gasCostInWeiRead.toString(), 'ether');
    console.log(`Cost of the Read transaction: ${gasCostInEtherRead* 1_000_000_000.0} gwei`);
    const tx1 = await KeyValueStoreInstance.Read(addr,{
      gas: gasEstimateRead,
      gasPrice: currentGasPrice 
    });
    assert(tx1);

    // Delete
    const gasEstimateDelete = await KeyValueStoreInstance.Delete.estimateGas(addr);
    console.log(`\nGas estimate for Delete transaction: ${gasEstimateDelete}`);
    const gasCostInWeiDelete = BigInt(gasEstimateDelete) * BigInt(currentGasPrice);
    const gasCostInEtherDelete = web3.utils.fromWei(gasCostInWeiDelete.toString(), 'ether');
    console.log(`Cost of the Delete transaction: ${gasCostInEtherDelete* 1_000_000_000.0} gwei`);
    const tx2 = await KeyValueStoreInstance.Delete(addr,{
      gas: gasEstimateDelete,
      gasPrice: currentGasPrice
    });
    assert(tx2);

    // sendEther
    const gasEstimateSendEther = await KeyValueStoreInstance.sendEther.estimateGas(addr);
    console.log(`\nGas estimate for sendEther transaction: ${gasEstimateSendEther}`);
    const gasCostInWeiSendEther = BigInt(gasEstimateSendEther) * BigInt(currentGasPrice);
    const gasCostInEtherSendEther = web3.utils.fromWei(gasCostInWeiSendEther.toString(), 'ether');
    console.log(`Cost of the sendEther transaction: ${gasCostInEtherSendEther* 1_000_000_000.0} gwei`);
    const tx3 = await KeyValueStoreInstance.sendEther(addr,{
      gas: gasEstimateSendEther,
      gasPrice: currentGasPrice 
    });
    assert(tx3);
  });
});
*/

// ---------------------------------------
// File: KeyValueStore.js
// Date: 01 Sept 2023
// Description: Performance Evaluation 
//              (Network latency)
// ---------------------------------------
const KeyValueStore = artifacts.require("KeyValueStore");

contract("KeyValueStore", () => {
  it("...should deploy and successfully call createInstance using the method's provided gas estimate", async () => {
    const KeyValueStoreInstance = await KeyValueStore.new();
    const currentGasPrice = await web3.eth.getGasPrice();
    const networkId = await web3.eth.net.getId();
    console.log(`Connected to network ID ${networkId}, Gas price: ${currentGasPrice} wei`);

    // random test parameters
    //const cipher = new Uint8Array(1000).fill(0x01); // 1k-byte ciphertext
    const cipher = "0x01"; // one-byte ciphertext
    const iv = "0x1234567890abcdef12345678";      
    const addr = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    const id = ["12345678"];

    // Write
    const gasEstimateWrite = await KeyValueStoreInstance.Write.estimateGas(cipher, iv, addr, id);
    const sendTimeWrite = new Date(); // Time when the transaction was sent
    const tx0 = await KeyValueStoreInstance.Write(cipher, iv, addr, id,{
      gas: gasEstimateWrite,
      gasPrice: currentGasPrice 
    });
    assert(tx0);
    const receivedTimeWrite = new Date(); // Time when the transaction was received
    const formattedTimeWrite = `${receivedTimeWrite.getHours()-sendTimeWrite.getHours()}:${receivedTimeWrite.getMinutes()-sendTimeWrite.getMinutes()}:${receivedTimeWrite.getSeconds()-sendTimeWrite.getSeconds()}:${receivedTimeWrite.getMilliseconds()-sendTimeWrite.getMilliseconds()}`;
    console.log(`Write transaction network latency: ${formattedTimeWrite}`);

    // Read
    const gasEstimateRead = await KeyValueStoreInstance.Read.estimateGas(addr);
    const sendTimeRead = new Date(); // Time when the transaction was sent
    const tx1 = await KeyValueStoreInstance.Read(addr,{
      gas: gasEstimateRead,
      gasPrice: currentGasPrice 
    });
    assert(tx1);
    const receivedTimeRead = new Date(); // Time when the transaction was received
    const formattedTimeRead = `${receivedTimeRead.getHours()-sendTimeRead.getHours()}:${receivedTimeRead.getMinutes()-sendTimeRead.getMinutes()}:${receivedTimeRead.getSeconds()-sendTimeRead.getSeconds()}:${receivedTimeRead.getMilliseconds()-sendTimeRead.getMilliseconds()}`;
    console.log(`Read transaction network latency: ${formattedTimeRead}`);

    // Delete
    const gasEstimateDelete = await KeyValueStoreInstance.Delete.estimateGas(addr);
    const sendTimeDelete = new Date(); // Time when the transaction was sent
    const tx2 = await KeyValueStoreInstance.Delete(addr,{
      gas: gasEstimateDelete,
      gasPrice: currentGasPrice
    });
    assert(tx2);
    const receivedTimeDelete = new Date(); // Time when the transaction was received
    const formattedTimeDelete = `${receivedTimeDelete.getHours()-sendTimeDelete.getHours()}:${receivedTimeDelete.getMinutes()-sendTimeDelete.getMinutes()}:${receivedTimeDelete.getSeconds()-sendTimeDelete.getSeconds()}:${receivedTimeDelete.getMilliseconds()-sendTimeDelete.getMilliseconds()}`;
    console.log(`Delete transaction network latency: ${formattedTimeDelete}`);

    // sendEther
    const gasEstimateSendEther = await KeyValueStoreInstance.sendEther.estimateGas(addr);
    const sendTimeSendEther = new Date(); // Time when the transaction was sent
    const tx3 = await KeyValueStoreInstance.sendEther(addr,{
      gas: gasEstimateSendEther,
      gasPrice: currentGasPrice 
    });
    assert(tx3);
    const receivedTimeSendEther = new Date(); // Time when the transaction was received
    const formattedTimeSendEther = `${receivedTimeSendEther.getHours()-sendTimeSendEther.getHours()}:${receivedTimeSendEther.getMinutes()-sendTimeSendEther.getMinutes()}:${receivedTimeSendEther.getSeconds()-sendTimeSendEther.getSeconds()}:${receivedTimeSendEther.getMilliseconds()-sendTimeSendEther.getMilliseconds()}`;
    console.log(`sendEther transaction network latency: ${formattedTimeSendEther}`);
  });
});