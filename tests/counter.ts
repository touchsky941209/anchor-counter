import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Counter } from "../target/types/counter";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";

describe("counter", () => {
  const adminProvider = anchor.AnchorProvider.env();
  const user1 = Keypair.generate()
  const user2 = Keypair.generate()
 
  const user1Provider = new anchor.AnchorProvider(
    adminProvider.connection,
    new anchor.Wallet(user1),
    adminProvider.opts
  )

  anchor.setProvider(user1Provider)
  // anchor.setProvider(adminProvider)


  const program = anchor.workspace.Counter as Program<Counter>;

  const [counterPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("counter")],
    program.programId,
  );

  it("airdrop ",async () => {
    const sign1 = await adminProvider.connection.requestAirdrop(user1.publicKey, LAMPORTS_PER_SOL)
    const sign2 = await adminProvider.connection.requestAirdrop(user2.publicKey, LAMPORTS_PER_SOL)
    await adminProvider.connection.confirmTransaction(sign1)
    await adminProvider.connection.confirmTransaction(sign2)
  })

  // it("Is initialized!", async () => {
  //   try {
  //     const txSig = await program.methods
  //       .initialize()
  //       .accounts({
  //         counter: counterPDA,
  //       })
  //       .rpc();

  //     const accountData = await program.account.counter.fetch(counterPDA);
  //     console.log("init success")
  //     console.log(`Count: ${accountData.count}`);
  //   } catch (error) {
  //     // If PDA Account already created, then we expect an error
  //     console.log("already init");
  //   }
  // });

  it("fetch account", async () => {
    const accountData = await program.account.counter.fetch(counterPDA);
    console.log(`Count: ${accountData.count}`)
  })


  it("Increment with admin provider", async () => {
    const transactionSignature = await program.methods
      .increment()
      .accounts({
        counter: counterPDA,
      })
      .rpc();

    const accountData = await program.account.counter.fetch(counterPDA);

    console.log(`Transaction Signature: ${transactionSignature}`);
    console.log(`Count: ${accountData.count}`);
  });

  // it("Increment with user1 provider", async () => {

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


  // it("Increment with user2 provider", async () => {

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
});