import * as anchor from "@coral-xyz/anchor";
import {
    PublicKey,
    Transaction,
    ComputeBudgetProgram,
    Keypair
} from "@solana/web3.js";
import {
    getAccount,
    getMint
} from "@solana/spl-token"
import {
    BN,
    Program,
} from "@coral-xyz/anchor";

import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
    createAssociatedTokenAccountIdempotentInstruction,
    createBurnInstruction,
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction
} from "@solana/spl-token"


export const createTransaction = () => {
    const transaction = new Transaction();
    transaction.add(
        ComputeBudgetProgram.setComputeUnitLimit({
            units: 200000
        })
    );
    return transaction;
}


export const transfer = async (
    provider: any,
    program: any,
    MINT_ADDRESS: PublicKey,
    FROM_ADDRESS: PublicKey,
    TO_ADDRESS: PublicKey,
    amount: number,
    programStandard: PublicKey

) => {
    const transaction = createTransaction();
    const senderAta = getAssociatedTokenAddressSync(
        MINT_ADDRESS,
        FROM_ADDRESS,
        true
    );
    const toAta = getAssociatedTokenAddressSync(
        MINT_ADDRESS,
        TO_ADDRESS,
        true
    );

    const toAtaInstruction =
    createAssociatedTokenAccountIdempotentInstruction(
        FROM_ADDRESS,
        toAta,
        TO_ADDRESS,
        MINT_ADDRESS,
        programStandard
    );
    
    transaction.add(toAtaInstruction);

    let send_amount = amount * 10 ** 9;

    transaction.add(
        await program.methods
            .tokenTransfer(new anchor.BN(send_amount))
            .accounts({
                mintToken: MINT_ADDRESS,
                userToken: senderAta,
                // tokenVault: TO_ADDRESS,
                tokenVaultAta:toAta,
            })
            .instruction()
    );

    const tx = await provider.sendAndConfirm(transaction)
    return tx
}

export const token_withdraw = async (
    provider: any,
    program: any,
    MINT_ADDRESS: PublicKey,
    USER_ADDRESS: PublicKey,
    TOKEN_VAULT_ADDRESS: PublicKey,
    amount: number,
    programStandard: PublicKey

) => {
    const transaction = createTransaction();


    const tokenVaultAta = getAssociatedTokenAddressSync(
        MINT_ADDRESS,
        TOKEN_VAULT_ADDRESS,
        false,
        programStandard
    );

    console.log("token ata =>", tokenVaultAta.toBase58())
    const tokenVaultAtaInstruction =
    createAssociatedTokenAccountIdempotentInstruction(
        TOKEN_VAULT_ADDRESS,
        tokenVaultAta,
        TOKEN_VAULT_ADDRESS,
        MINT_ADDRESS,
        programStandard
    );
    
    transaction.add(tokenVaultAtaInstruction);
    
    const userAta = getAssociatedTokenAddressSync(
        MINT_ADDRESS,
        USER_ADDRESS,
        true,
        programStandard
    );
    console.log("user ata =>", userAta.toBase58())

    const userAtaInstruction =
        createAssociatedTokenAccountIdempotentInstruction(
            USER_ADDRESS,
            userAta,
            USER_ADDRESS,
            MINT_ADDRESS,
            programStandard
        );

    transaction.add(userAtaInstruction);

    const mint = await provider.connection.getTokenSupply(MINT_ADDRESS);
    const decimals = mint.value.decimals;
    // Fix: Ensure proper BN calculation
    const multiplier = new BN(10).pow(new BN(decimals));
    // const sendAmount = new BN(SEND_AMOUNT).mul(multiplier);
    let send_amount = amount * 10 ** decimals;

    transaction.add(
        await program.methods
            .tokenWithdraw(new anchor.BN(send_amount))
            .accounts({
                mintToken: MINT_ADDRESS,
                userToken: userAta,
                tokenVault: tokenVaultAta,
                signer:program.provider.publicKey,
            })
            .instruction()
    );

    const tx = await provider.sendAndConfirm(transaction)
    return tx
}