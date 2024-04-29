import * as util from './util.js';
import { Keyring } from '@polkadot/keyring';

export async function testAll() {
    console.log("Starting Polkadot JS Test.");
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
    console.log(`Dave's request of Nft data: ${(await api.query.nft.pendingNft(nft_id))}`);

    // Auditor approve the Nft.
    await util.sendExtrinsicAndWait(api.tx.nft.approveNft(nft_id, Response.Accept), auditor);
    // Verify the Nft is been approved.
    console.log(`Auditor approved the Nft: ${(await api.query.nft.owners(nft_id))}`);

    // Eve create a Nft request.
    // Get the Nft id.
    let nft_id_1 = Number(await api.query.nft.nextNftId()) + 1;
    console.log(`Next nft id is ${nft_id_1}`);

    let data_1 = [0x4E, 0x46, 0x54, 0x31];
    let file_name_1 = [0x46, 0x49, 0x4C, 0x45, 0x31];
    await util.sendExtrinsicAndWait(api.tx.nft.requestMint(file_name_1, data_1), eve);
    // Verify the request is successful.
    console.log(`Eve's request of Nft data: ${(await api.query.nft.pendingNft(nft_id_1))}`);

    // Auditor approved.
    await util.sendExtrinsicAndWait(api.tx.nft.approveNft(nft_id_1, Response.Accept), auditor);
    // Verify the Nft is been approved.
    console.log(`Auditor approved the Nft: ${(await api.query.nft.owners(nft_id_1))}`);

    // Dave create a POD to Eve. Verify balance that tax is paid.
    let pod_id = Number(await api.query.nft.nextPodId()) + 1;
    console.log(`Dave sent to Eve's pod id is ${pod_id}`);

    let price = util.fromDollar(30);
    let tax = util.toDollar(api.consts.nft.podFee.toNumber());
    console.log(`Dave's free balance : $${util.toDollar((await api.query.bank.accounts(dave.publicKey)).free.toString())}`);
    console.log(`The tax of Create POD is : $${tax}`);
    await util.sendExtrinsicAndWait(api.tx.nft.createPod(eve.publicKey, nft_id, price), dave);
    console.log(`Dave's free balance after tax paid: $${util.toDollar((await api.query.bank.accounts(dave.publicKey)).free.toString())}`);

    // Eve create a POD to Charlie. Verify balance that tax is paid.
    let pod_id_1 = Number(await api.query.nft.nextPodId()) + 1;
    console.log(`Eve sent to Charlie's pod id is ${pod_id_1}`);

    let price_1 = util.fromDollar(50);

    console.log(`Eve's free balance : $${util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString())}`);
    console.log(`The tax of Create POD is : $${tax}`);
    await util.sendExtrinsicAndWait(api.tx.nft.createPod(charlie.publicKey, nft_id_1, price_1), eve);
    console.log(`Eve's free balance after tax paid: $${util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString())}`);

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

    console.log(`Before paid the Nft is belong to Eve : ${(await api.query.nft.owners(nft_id_1))}`);
    console.log(`Charlie's free balance : $${util.toDollar((await api.query.bank.accounts(charlie.publicKey)).free.toString())}`);
    console.log(`The price of the Nft POD is : $${util.toDollar(price_1)}`);
    console.log(`The tips of the Nft POD is : $${util.toDollar(tips)}`);

    await util.sendExtrinsicAndWait(api.tx.nft.receivePod(pod_id_1, Response.Accept, tips), charlie);

    console.log(`Charlie's free balance after buying Nft and giving tips : $${util.toDollar((await api.query.bank.accounts(charlie.publicKey)).free.toString())}`);
    console.log(`After paid the Nft is belong to Charlie : ${(await api.query.nft.owners(nft_id_1))}`);

    // Verify Rpc call pending pods that Eve has one receiving.
    let nftPodData_eve_received = await api.rpc.xyChain.pending_pods(eve.publicKey);
    console.log(`Eve RPC Nft pending data: ${nftPodData_eve_received}`);

    // Eve rejected POD from Dave and verify owner of Nft is Dave, Eve is not be charged.
    console.log(`Eve's free balance before rejecting Nft Pod : $${util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString())}`);
    console.log(`Before rejected the Nft is belong to Dave : ${(await api.query.nft.owners(nft_id))}`);

    await util.sendExtrinsicAndWait(api.tx.nft.receivePod(pod_id, Response.Reject, 0), eve);

    console.log(`Eve's free balance after rejecting Nft Pod : $${util.toDollar((await api.query.bank.accounts(eve.publicKey)).free.toString())}`);
    console.log(`After rejected the Nft is belong to Dave : ${(await api.query.nft.owners(nft_id))}`);

    // Verify Rpc call pending pods of Eve is null.
    let nftPodData_eve_none = await api.rpc.xyChain.pending_pods(eve.publicKey);
    console.log(`Eve RPC Nft pending data: ${nftPodData_eve_none}`);

}