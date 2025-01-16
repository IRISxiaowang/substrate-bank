import * as util from './util.js';
import { Keyring } from '@polkadot/keyring';
import * as assert from 'assert';

export async function testAll() {
    console.log("Starting Polkadot JS Test NFT functionalities.");
    let api = await util.getApi();

    await testNftFunctionalities(api);

    await api.disconnect();
}

async function testNftFunctionalities(api) {
    let keyring = new Keyring({ type: 'sr25519' });
    let auditor = keyring.addFromUri('//Bob');
    let charlie = keyring.addFromUri('//Charlie');
    let dave = keyring.addFromUri('//Dave');
    let eve = keyring.addFromUri('//Eve');
    let data = [0x4E, 0x46, 0x54];
    let file_name = [0x46, 0x49, 0x4C, 0x45];
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
    console.log(`Dave's request of Nft data: ${dave_pending_nft}`);

    assert.ok(parsed[0].fileName == "0x46494c45" );

    // Auditor approve the Nft.
    await util.sendExtrinsicAndWait(api.tx.nft.approveNft(nft_id, Response.Accept), auditor);
    // Verify the Nft is been approved.
    let owner = await api.query.nft.owners(nft_id);
    console.log(`Auditor approved Dave's Nft: ${owner}`);
    console.log(`Dave's Nft: ${util.bytesArrayToPolkadotAddress(dave.publicKey)}`);
    
    assert.ok(owner == dave.address);

    // Eve create a Nft request.
    // Get the Nft id.
    let nft_id_1 = Number(await api.query.nft.nextNftId()) + 1;
    console.log(`Next nft id is ${nft_id_1}`);

    let data_1 = [0x4E, 0x46, 0x54, 0x31];
    let file_name_1 = [0x46, 0x49, 0x4C, 0x45, 0x31];
    await util.sendExtrinsicAndWait(api.tx.nft.requestMint(file_name_1, data_1), eve);
    // Verify the request is successful.
    let eve_pending_nft = await api.query.nft.pendingNft(nft_id_1);
    let parsed_eve = JSON.parse(eve_pending_nft);
    console.log(`Eve's request of Nft data: ${eve_pending_nft}`);

    assert.ok(parsed_eve[0].fileName == "0x46494c4531");

    // Auditor approved.
    await util.sendExtrinsicAndWait(api.tx.nft.approveNft(nft_id_1, Response.Accept), auditor);
    // Verify the Nft is been approved.
    let owner_eve = await api.query.nft.owners(nft_id_1);
    console.log(`Auditor approved Eve's Nft: ${owner_eve}`);

    assert.ok(owner_eve == eve.address);

    // Dave create a POD to Eve. Verify balance that tax is paid.
    let pod_id = Number(await api.query.nft.nextPodId()) + 1;
    console.log(`Dave sent to Eve's pod id is ${pod_id}`);

    let price = util.fromDollar(30);
    let tax = util.toDollar(api.consts.nft.podFee.toNumber());
    
    let dave_free = util.toDollar((await api.query.bank.accounts(dave.publicKey)).free.toString());
    console.log(`Dave's free balance : $${dave_free}`);
    console.log(`The tax of Create POD is : $${tax}`);
    await util.sendExtrinsicAndWait(api.tx.nft.createPod(eve.publicKey, nft_id, price), dave);

    let dave_free_after_tax = util.toDollar((await api.query.bank.accounts(dave.publicKey)).free.toString());
    console.log(`Dave's free balance after tax paid: $${dave_free_after_tax}`);

    assert.ok(dave_free_after_tax == dave_free - tax);

    // Eve create a POD to Charlie. Verify balance that tax is paid.
    let pod_id_1 = Number(await api.query.nft.nextPodId()) + 1;
    console.log(`Eve sent to Charlie's pod id is ${pod_id_1}`);

    let price_1 = util.fromDollar(50);

    let eve_free = util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString());
    console.log(`Eve's free balance : $${eve_free}`);
    console.log(`The tax of Create POD is : $${tax}`);
    await util.sendExtrinsicAndWait(api.tx.nft.createPod(charlie.publicKey, nft_id_1, price_1), eve);
    let eve_free_after_tax = util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString());
   
    console.log(`Eve's free balance after tax paid: $${eve_free_after_tax}`);

    assert.ok(eve_free_after_tax == eve_free - tax);

    // Test Rpc call pending pods that Eve has one delivering one receiving, 
    // Charlie has one receiving and Dave has one delivering
    let nftPodData_charlie = await api.rpc.xyChain.pending_pods(charlie.publicKey);
    console.log(`Charlie RPC Nft pending data: ${nftPodData_charlie}`);
    let nftPodData_dave = await api.rpc.xyChain.pending_pods(dave.publicKey);
    console.log(`Dave RPC Nft pending data: ${nftPodData_dave}`);
    let nftPodData_eve = await api.rpc.xyChain.pending_pods(eve.publicKey);
    console.log(`Eve RPC Nft pending data: ${nftPodData_eve}`);

    // Charlie accepted POD and verify the owner of Nft and balance of Charlie.
    let tips = util.fromDollar(10);

    let eve_free_before = util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString());
    let charlie_free_before = util.toDollar((await api.query.bank.accounts(charlie.publicKey)).free.toString());

    let nft_1_owner = await api.query.nft.owners(nft_id_1);
    console.log(`Before paid the Nft is belong to Eve : ${nft_1_owner}`);

    assert.ok(nft_1_owner == eve.address);

    console.log(`Charlie's free balance : $${charlie_free_before}`);
    console.log(`The price of the Nft POD is : $${util.toDollar(price_1)}`);
    console.log(`The tips of the Nft POD is : $${util.toDollar(tips)}`);

    await util.sendExtrinsicAndWait(api.tx.nft.receivePod(pod_id_1, Response.Accept, tips), charlie);

    let nft_1_owner_changed = await api.query.nft.owners(nft_id_1);

    console.log(`Charlie's free balance after buying Nft and giving tips : $${util.toDollar((await api.query.bank.accounts(charlie.publicKey)).free.toString())}`);
    console.log(`After paid the Nft is belong to Charlie : ${nft_1_owner_changed}`);

    let eve_free_after = util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString());
    let charlie_free_after = util.toDollar((await api.query.bank.accounts(charlie.publicKey)).free.toString());

    assert.ok(nft_1_owner_changed == charlie.address);
    assert.ok(eve_free_before == eve_free_after - util.toDollar(price_1) - util.toDollar(tips));
    assert.ok(charlie_free_before == charlie_free_after + util.toDollar(price_1) + util.toDollar(tips));


    // Verify Rpc call pending pods that Eve has one receiving.
    let nftPodData_eve_received = await api.rpc.xyChain.pending_pods(eve.publicKey);
    console.log(`Eve RPC Nft pending data: ${nftPodData_eve_received}`);

    // Eve rejected POD from Dave and verify owner of Nft is Dave, Eve is not be charged.
    let dave_free_before = util.toDollar((await api.query.bank.accounts(charlie.publicKey)).free.toString());
    let nft_owner = await api.query.nft.owners(nft_id);
    console.log(`Eve's free balance before rejecting Nft Pod : $${util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString())}`);
    console.log(`Before rejected the Nft is belong to Dave : ${nft_owner}`);

    assert.ok(nft_owner == dave.address);

    await util.sendExtrinsicAndWait(api.tx.nft.receivePod(pod_id, Response.Reject, 0), eve);

    let dave_free_after = util.toDollar((await api.query.bank.accounts(charlie.publicKey)).free.toString());

    console.log(`Eve's free balance after rejecting Nft Pod : $${util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString())}`);
    console.log(`After rejected the Nft is belong to Dave : ${(await api.query.nft.owners(nft_id))}`);

    assert.ok(dave_free_before == dave_free_after);
    assert.ok(nft_owner == dave.address);
    
    // Verify Rpc call pending pods of Eve is null.
    let nftPodData_eve_none = await api.rpc.xyChain.pending_pods(eve.publicKey);
    console.log(`Eve RPC Nft pending data: ${nftPodData_eve_none}`);
   
    assert.ok(nftPodData_eve_none.delivering.length == 0);

    console.log(`tests finished ok`);

}