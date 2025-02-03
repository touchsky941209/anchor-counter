import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Counter } from "../target/types/counter";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token"
import { transfer, token_withdraw } from "./token_control"
describe("counter", () => {
  const adminProvider = anchor.AnchorProvider.env();
  const user1 = Keypair.fromSecretKey(Uint8Array.from(require('./us1hbhq71B8B865ARa3NkYfKn8fwn771bgMZdCdzyiy.json')))
  const user2 = Keypair.fromSecretKey(Uint8Array.from(require('./us2P2tY6Tf8T5cacUJ8NzmA6mmxS2bDh7onftgGJaty.json')))
  const user3 = Keypair.fromSecretKey(Uint8Array.from(require('./us3LpDfaodp916sYrQdW13jbKT2ptZjNdb3Hcm6xvPu.json')))
  const mintToken = Keypair.fromSecretKey(Uint8Array.from(require('./mntrKJfjURt4LFq7VF6RxkzprQwSbRvp9aN3WAM4JNf.json')));

  const user1Provider = new anchor.AnchorProvider(
    adminProvider.connection,
    new anchor.Wallet(user1),
    adminProvider.opts
  )

  // anchor.setProvider(user1Provider)
  anchor.setProvider(adminProvider)


  const program = anchor.workspace.Counter as Program<Counter>;

  const [counterPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("counter")],
    program.programId,
  );
  const [tokenVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("token_vault")],
    program.programId,
  );

  it("airdrop ", async () => {
    const sign1 = await adminProvider.connection.requestAirdrop(user1.publicKey, LAMPORTS_PER_SOL)
    const sign2 = await adminProvider.connection.requestAirdrop(user2.publicKey, LAMPORTS_PER_SOL)
    await adminProvider.connection.confirmTransaction(sign1)
    await adminProvider.connection.confirmTransaction(sign2)
  })

  it("Is initialized!", async () => {
    try {
      const txSig = await program.methods
        .initialize()
        .accounts({
          counter: counterPDA,
          tokenVault: tokenVaultPda
        })
        .rpc();

      const accountData = await program.account.counter.fetch(counterPDA);
      console.log("init success")
      console.log(`Count: ${accountData.count}`);
    } catch (error) {
      // If PDA Account already created, then we expect an error
      console.log("already init",error);
    }
  });

  // it("fetch account", async () => {
  //   const accountData = await program.account.counter.fetch(counterPDA);
  //   console.log(`Count: ${accountData.count}`)
  // })


  // it("Increment with admin provider", async () => {
  //   const transactionSignature = await program.methods
  //     .increment()
  //     .accounts({
  //       counter: counterPDA,
  //     })
  //     .rpc();

  //   const accountData = await program.account.counter.fetch(counterPDA);

  //   console.log(`Transaction Signature: ${transactionSignature}`);
  //   console.log(`Count: ${accountData.count}`);
  // });

  it("Token transfer", async () => {
    const amount = 100000
    const provider = adminProvider
    const programStandard = TOKEN_PROGRAM_ID;
    const MINT_ADDRESS = mintToken.publicKey
    const FROM_ADDRESS = provider.wallet.publicKey
    const [tokenVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_vault")],
      program.programId,
    );


    const TO_ADDRESS = tokenVaultPda

    console.log("pda =>", tokenVaultPda.toBase58())
    const tx = await transfer(
      provider,
      program,
      MINT_ADDRESS,
      FROM_ADDRESS,
      TO_ADDRESS,
      amount,
      programStandard
    )
  })

  // it("Token withdraw", async () => {
  //   const amount = 10
  //   const provider = adminProvider
  //   const programStandard = TOKEN_PROGRAM_ID;
  //   const MINT_ADDRESS = mintToken.publicKey
  //   const FROM_ADDRESS = adminProvider.publicKey
  //   const TO_ADDRESS = program.programId





  //   const tx = await token_withdraw(
  //     provider,
  //     program,
  //     MINT_ADDRESS,
  //     FROM_ADDRESS,
  //     TO_ADDRESS,
  //     amount,
  //     programStandard
  //   )
  // })

});