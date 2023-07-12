//migration file

var KeyValueStore = artifacts.require("./contracts/keyValueStore.sol");

module.exports = function (deployer) {
  deployer.deploy(KeyValueStore);
};