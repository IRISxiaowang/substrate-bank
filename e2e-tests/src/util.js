// Import
import { ApiPromise, WsProvider } from '@polkadot/api';
import { encodeAddress } from '@polkadot/util-crypto';

export async function getApi() {
    const api = await ApiPromise.create({
        provider: new WsProvider('ws://127.0.0.1:9944'),
        rpc: {
            xyChain: {
                account_data: {
                    description: 'For getting Account Data of a user.',
                    params: [
                        {
                            name: 'who',
                            type: 'AccountId'
                        },
                    ],
                    type: 'RpcAccountData',
                },
                interest_pa: {
                    description: 'Estimates interest yearned per year for a user.',
                    params: [
                        {
                            name: 'who',
                            type: 'AccountId'
                        },
                    ],
                    type: 'String',
                },
                pending_pods: {
                    description: 'For getting related Nft in POD info for a user.',
                    params: [
                        {
                            name: 'who',
                            type: 'AccountId'
                        },
                    ],
                    type: 'PendingNftPods',
                }
            }
        },
        types: {
            RpcAccountData: {
                free: 'String',
                reserved: 'String',
                locked: 'Vec<RpcLockedFund>',
            },
            RpcLockedFund: {
                id: 'LockId',
                amount: 'String',
                reason: 'LockReason',
                unlock_at: 'BlockNumber',
            },
            LockId: 'u64',
            LockReason: {
                _enum: ['Stake', 'Redeem', 'Auditor',],
            },
            PendingNftPods: {
                delivering: 'Vec<RpcNftData>',
                receiving: 'Vec<RpcNftData>',
            },
            RpcNftData: {
                pod_id: 'PodId',
                sender: 'AccountId',
                nft_id: 'NftId',
                nft_name: 'Vec<u8>',
                expiry_block: 'BlockNumber',
                price: 'String',
            },
            PodId: 'u32',
            NftId: 'u32',
            BlockNumber: 'u32'
        }
    });
    // Wait until we are ready and connected
    await api.isReady;
    return api;
}

export function toDollar(input) {
    return input * 1.0 / 1000000000000
}

export function fromDollar(input) {
    return input * 1000000000000
}

export function toDay(input) {
    return input * 1.0 / (10 * 60 * 24)
}

export async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export async function sendExtrinsicAndWait(extrinsic, signer) {
    let finished = false;
    let unsub = await extrinsic.signAndSend(signer, (result) => {
        if (result.status.isInBlock) {
            unsub();
            finished = true;
        }
    });
    while (!finished) {
        await sleep(500);
    }
} 

export function bytesArrayToPolkadotAddress(byteArray) {
    // Convert the byte array to a Uint8Array
    const byteU8Array = new Uint8Array(byteArray);

    // Encode the Uint8Array to a Polkadot address
    const polkadotAddress = encodeAddress(byteU8Array, 42); // The second argument is the SS58 prefix, 0 is for Polkadot

    return polkadotAddress;
}