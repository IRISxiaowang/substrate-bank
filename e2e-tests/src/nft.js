import * as util from './util.js';
import { Keyring } from '@polkadot/keyring';
import * as assert from 'assert';
import fs from 'fs';
import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import { u8aToHex } from '@polkadot/util';


export async function testAll() {
    console.log("Starting Polkadot JS Test NFT download.");

    let api = await util.getApi();

    await testNftDataDownload(api);

    await api.disconnect();
}

async function testNftDataDownload(api) {
    // Convert `import.meta.url` to a directory path
    const filePath = fileURLToPath(import.meta.url);
    const dirPath = dirname(filePath);

    let keyring = new Keyring({ type: 'sr25519' });
    let auditor = keyring.addFromUri('//Bob');
    let dave = keyring.addFromUri('//Dave');
    let data = u8aToHex(pngToBytes(dirPath + '/penguin.png'));
    let file_name = fileNameToBytes('penguin.png');

    // For js to read the enum in rust.
    const Response = {
        Accept: 0,
        Reject: 1,
    };

    // Get the Nft id.
    let nft_id = Number(await api.query.nft.nextNftId()) + 1;
    console.log(`Next nft id is ${nft_id}`);

    // Dave proposes an Nft create request.
    await util.sendExtrinsicAndWait(api.tx.nft.requestMint(file_name, data), dave);
    // Verify the request is successful.
    let dave_pending_nft = await api.query.nft.pendingNft(nft_id);
    let parsed = JSON.parse(dave_pending_nft);

    assert.ok(parsed[0].fileName == bytesToHex(file_name));

    // Auditor approve the Nft.
    await util.sendExtrinsicAndWait(api.tx.nft.approveNft(nft_id, Response.Accept), auditor);
    // Verify the Nft is been approved.
    let owner = await api.query.nft.owners(nft_id);
    console.log(`Auditor approved Dave's Nft: ${owner}`);
    console.log(`Dave's Nft: ${util.bytesArrayToPolkadotAddress(dave.publicKey)}`);

    assert.ok(owner == dave.address);

    // Test Rpc call 
    let nftData_dave = await api.rpc.xyChain.nft_data(nft_id);

    // Download the Nft image.
    hexToFile(nftData_dave.data, dirPath + '/output.png');
}

// Convert PNG file to u8 bytes
function pngToBytes(filePath) {
    try {
        const buffer = fs.readFileSync(filePath); // Read the file as a buffer
        const u8Bytes = new Uint8Array(buffer);  // Convert the buffer to Uint8Array
        // console.log('PNG file as u8 bytes:', u8Bytes);
        return u8Bytes;
    } catch (err) {
        console.error('Error reading the PNG file:', err);
    }
}

// Convert file name to u8 bytes
function fileNameToBytes(fileName) {
    const encoder = new TextEncoder(); // Built-in encoder for UTF-8
    const u8Bytes = encoder.encode(fileName); // Encode the string to Uint8Array
    // console.log('File name as u8 bytes:', u8Bytes);
    return Array.from(u8Bytes);
}
// Convert u8 bytes to a string
function bytesToHex(bytes) {
    const hex = bytes
        .map(byte => byte.toString(16).padStart(2, '0')) // Convert to hex and pad with 0 if needed
        .join('');

    // Add the "0x" prefix
    return `0x${hex}`;
}

function hexToFile(hexString, outputFilePath) {
    // Ensure the input is a string
    if (typeof hexString !== 'string') {
        hexString = String(hexString); // Convert to a string
    }

    if (hexString.startsWith('0x')) {
        hexString = hexString.slice(2);
    }

    // Ensure the hex string length is even
    if (hexString.length % 2 !== 0) {
        throw new Error('Invalid hex string: Length must be even.');
    }

    // Convert the hex string into a Uint8Array
    const byteArray = new Uint8Array(
        hexString.match(/.{1,2}/g).map(byte => parseInt(byte, 16))
    );

    // Write the binary data to a file
    writeFileSync(outputFilePath, byteArray);

    console.log(`File saved to ${outputFilePath}`);
}