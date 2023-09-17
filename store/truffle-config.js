// You should create your own .env file and enter the INFURA_API_KEY and MNEMONIC.
require('dotenv').config();
const HDWalletProvider = require('@truffle/hdwallet-provider');
const { INFURA_API_KEY, MNEMONIC } = process.env;

module.exports = {
  //plugins: [
  //  'truffle-plugin-verify'
  //],
  //api_keys: {
  //   etherscan: etherscan_api_key
  //},

  networks: {
    development: {
      host: "127.0.0.1",     // Localhost (default: none)
      port: 9545,            // Standard Ethereum port (default: none)
      network_id: "*",       // Any network (default: none)
      websocket: true
    },

    //sepolia: {
    //  provider: () => new HDWalletProvider(MNEMONIC, INFURA_API_KEY),
    //  network_id: "11155111",
    //  gas: 4465030, 
    //},

    //goerli: {
    //  provider: () => new HDWalletProvider(MNEMONIC, INFURA_API_KEY),
    //  network_id: 5, //Goerli's id
    //  gas: 5000000, //gas limit
    //},
  },

  mocha: {
    // timeout: 100000
  },
  
  // Configure your compilers
  compilers: {
    solc: {
      version: "0.8.15",    // Fetch exact version from solc-bin (default: truffle's version)
      //settings: {          // See the solidity docs for advice about optimization and evmVersion
      //  optimizer: {
      //    enabled: true,
      //    runs: 200
      //  },
      //  evmVersion: "istanbul"
      //}
    }
  },
};