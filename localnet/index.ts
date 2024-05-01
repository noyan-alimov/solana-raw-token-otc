import { Connection, Keypair, PublicKey, TransactionInstruction, TransactionMessage, VersionedTransaction } from '@solana/web3.js'
import fs from 'fs'

const bytesKp = fs.readFileSync('/Users/noyan/.config/solana/id.json')
const keypair = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(bytesKp.toString())))

const connection = new Connection('http://127.0.0.1:8899', 'confirmed')
const programId = new PublicKey('RguigFdFWUa2oSDGSD66LC22YWEq11GJ4MbnZSs7Fsk')

const ixn = new TransactionInstruction({
    keys: [{
        pubkey: keypair.publicKey,
        isSigner: false,
        isWritable: false
    }],
    programId,
    data: Buffer.alloc(0)
})

const messageV0 = new TransactionMessage({
    payerKey: keypair.publicKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
    instructions: [ixn]
}).compileToV0Message()
const txnV0 = new VersionedTransaction(messageV0)
txnV0.sign([keypair])

connection.sendTransaction(txnV0).then(console.log).catch(console.error)
