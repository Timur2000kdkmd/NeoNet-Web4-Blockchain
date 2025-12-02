const hre = require('hardhat');

async function main() {
  const [deployer] = await hre.ethers.getSigners();
  console.log('Deploying contracts with account:', deployer.address);

  const NeoToken = await hre.ethers.getContractFactory('NeoToken');
  const neo = await NeoToken.deploy(hre.ethers.utils.parseEther('1000000'));
  await neo.deployed();
  console.log('NeoToken deployed to', neo.address);

  const Stake = await hre.ethers.getContractFactory('StakeAndVote');
  const stake = await Stake.deploy(neo.address);
  await stake.deployed();
  console.log('StakeAndVote deployed to', stake.address);

  const Oracle = await hre.ethers.getContractFactory('Oracle');
  const oracle = await Oracle.deploy();
  await oracle.deployed();
  console.log('Oracle deployed to', oracle.address);

  let tx = await stake.setOracle(oracle.address);
  await tx.wait();
  console.log('Oracle set in StakeAndVote');

  const accounts = await hre.ethers.getSigners();
  for (let i = 1; i < Math.min(accounts.length, 5); i++) {
    await neo.transfer(accounts[i].address, hre.ethers.utils.parseEther('1000'));
    console.log('Transferred tokens to', accounts[i].address);
  }

  console.log(JSON.stringify({
    neo: neo.address,
    stake: stake.address,
    oracle: oracle.address
  }, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
