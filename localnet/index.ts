import { TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { Connection, Keypair, PublicKey, SystemProgram, TransactionInstruction, TransactionMessage, VersionedTransaction } from '@solana/web3.js'
import fs from 'fs'
import BN from 'bn.js'

const bytesKp = fs.readFileSync('/Users/noyan/.config/solana/id.json')
const keypair = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(bytesKp.toString())))

const connection = new Connection('http://127.0.0.1:8899', 'confirmed')
const programId = new PublicKey('Fncs4u93Uvneur4CGV19pirRWwu5Kzbyyoi8MHU1JDnP')

const offeredMint = new PublicKey('9umHeVn7PxGiRC4RC4MMNx8dg1eAQHFhyBddqUp3frL4')
const ataCreatorOffered = new PublicKey('J4nV2YGRVRvdgmYEYwnAsVCHGCJvNtm5WXisNAi8qH5S')
const desiredMint = new PublicKey('AGAkapXW8tYP8sxDKqeiJGJfdpmEPqUSg7Q8Wj4titGL')
const swap = PublicKey.findProgramAddressSync([Buffer.from('swap'), ataCreatorOffered.toBuffer()], programId)[0]
const escrow = PublicKey.findProgramAddressSync([Buffer.from('escrow'), ataCreatorOffered.toBuffer()], programId)[0]

const DECIMAL_UNIT_PER_TOKEN = 1_000_000_000

const createSwapIxn = new TransactionInstruction({
    keys: [
        {
            pubkey: keypair.publicKey,
            isSigner: true,
            isWritable: true
        },
        {
            pubkey: offeredMint,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: ataCreatorOffered,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: desiredMint,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: swap,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: escrow,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false
        }
    ],
    programId,
    data: Buffer.from(Uint8Array.of(
        0, // CreateSwap instruction
        ...new BN(20 * DECIMAL_UNIT_PER_TOKEN).toArray('le', 8), // offeredAmount
        ...new BN(5 * DECIMAL_UNIT_PER_TOKEN).toArray('le', 8) // desiredAmount
    ))
})

const cancelSwapIxn = new TransactionInstruction({
    keys: [
        {
            pubkey: keypair.publicKey,
            isSigner: true,
            isWritable: true
        },
        {
            pubkey: ataCreatorOffered,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: swap,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: escrow,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false
        }
    ],
    programId,
    data: Buffer.from(Uint8Array.of(
        1, // CancelSwap instruction
    ))
})

const messageV0 = new TransactionMessage({
    payerKey: keypair.publicKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
    // instructions: [createSwapIxn]
    instructions: [cancelSwapIxn]
}).compileToV0Message()
const txnV0 = new VersionedTransaction(messageV0)
txnV0.sign([keypair])

connection.sendTransaction(txnV0).then(console.log).catch(console.error)
