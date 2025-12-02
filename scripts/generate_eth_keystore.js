// generate_eth_keystore.js
// Usage: node generate_eth_keystore.js <password> [count]
// Generates `count` Ethereum accounts and writes keystore JSON files and a keys.json manifest.
const { ethers } = require('ethers');
const fs = require('fs');
const path = require('path');

const password = process.argv[2];
const count = parseInt(process.argv[3] || '3', 10);
if (!password) {
  console.error('Usage: node generate_eth_keystore.js <password> [count]');
  process.exit(1);
}

const out = path.resolve('secrets/keystores');
if (!fs.existsSync(out)) fs.mkdirSync(out, { recursive: true });

const manifest = [];
for (let i = 0; i < count; i++) {
  const wallet = ethers.Wallet.createRandom();
  const json = await wallet.encrypt(password);
  const filename = path.join(out, `keystore-${i}.json`);
  fs.writeFileSync(filename, json);
  manifest.push({ address: wallet.address, keystore: filename });
  console.log('Generated', wallet.address, filename);
}
fs.writeFileSync(path.resolve('secrets/keystore_manifest.json'), JSON.stringify(manifest, null, 2));
console.log('Wrote secrets/keystore_manifest.json');
