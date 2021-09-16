import { AccountLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ESCROW_ACCOUNT_DATA_LAYOUT, EscrowLayout } from "./layout";
import idl from "../../idl/escrow_anchor.json";

import * as anchor from '@project-serum/anchor';
import {init} from "./wallet";

const {Account, SystemProgram, Transaction, SYSVAR_RENT_PUBKEY, PublicKey, Connection} = anchor.web3

const connection = new Connection("http://localhost:8899", 'singleGossip');

export const initEscrow = async (
    initializerXTokenAccountPubkeyString: string,
    amountXTokensToSendToEscrow: number,
    initializerReceivingTokenAccountPubkeyString: string,
    expectedAmount: number,
    escrowProgramIdString: string
  ) => {
    const provider: any = await init();
    provider.connection = connection;
    // Configure the client to use the local cluster.
    
    anchor.setProvider(provider)
    const escrowProgramId = new PublicKey(escrowProgramIdString);
    const program = new anchor.Program(idl as any, escrowProgramId, provider);

    const initializerAccountPubkey = new PublicKey(provider.publicKey);
    const initializerXTokenAccountPubkey = new PublicKey(initializerXTokenAccountPubkeyString);

    //@ts-expect-error
    const XTokenMintAccountPubkey = new PublicKey((await connection.getParsedAccountInfo(initializerXTokenAccountPubkey, 'singleGossip')).value!.data.parsed.info.mint);

    const tempTokenAccount = new Account();
    const createTempTokenAccountIx = SystemProgram.createAccount({
        programId: TOKEN_PROGRAM_ID,
        space: AccountLayout.span,
        lamports: await connection.getMinimumBalanceForRentExemption(AccountLayout.span, 'singleGossip'),
        fromPubkey: initializerAccountPubkey,
        newAccountPubkey: tempTokenAccount.publicKey
    });
    const initTempAccountIx = Token.createInitAccountInstruction(TOKEN_PROGRAM_ID, XTokenMintAccountPubkey, tempTokenAccount.publicKey, initializerAccountPubkey);
    const transferXTokensToTempAccIx = Token.createTransferInstruction(
      TOKEN_PROGRAM_ID,
      initializerXTokenAccountPubkey,
      tempTokenAccount.publicKey,
      initializerAccountPubkey,
      [],
      amountXTokensToSendToEscrow
    );
    
    const escrowAccount = new Account()

    // We also have to add the other two accounts because it turns out when the 
    // system program creates a new account, the tx needs to be signed by that account.
    // this is why signers array includes the escrowAccount
    const initEscrowIx = await program.transaction.initEscrow(new anchor.BN(expectedAmount), {
      accounts: {
        escrow: escrowAccount.publicKey,
        initializer: initializerAccountPubkey,
        tmpTokenAccount: tempTokenAccount.publicKey,
        tokenToReceiveAccount: new PublicKey(initializerReceivingTokenAccountPubkeyString),
        // rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      options: {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
        skipPreflight: true
      },
      signers: [escrowAccount]
    });
    const tx = new Transaction().add(
      createTempTokenAccountIx,
      initTempAccountIx,
      transferXTokensToTempAccIx,
      initEscrowIx
    );

    const {blockhash} = await connection.getRecentBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = initializerAccountPubkey;
    tx.partialSign(tempTokenAccount, escrowAccount);
    
    const signedTx = await provider.signTransaction(tx);
    
    const txHash = await connection.sendRawTransaction(
      signedTx.serialize(),
      {skipPreflight: false, preflightCommitment: 'singleGossip'}
    );

    console.log('Sent ', txHash)

    await new Promise((resolve) => setTimeout(resolve, 1000));

    let escrowAccountInfo: any;
    try {
      escrowAccountInfo = await program.account.escrow.fetch(escrowAccount.publicKey)
    } catch(error) {
      console.log((error as any).stack)
    }

    return {
        escrowAccountPubkey: escrowAccount.publicKey.toBase58(),
        isInitialized: !!escrowAccountInfo.isInitialized,
        initializerAccountPubkey: new PublicKey(escrowAccountInfo.initializerPubkey).toBase58(),
        XTokenTempAccountPubkey: new PublicKey(escrowAccountInfo.tmpTokenAccountPubkey).toBase58(),
        initializerYTokenAccount: new PublicKey(escrowAccountInfo.initializerTokenToReceiveAccountPubkey).toBase58(),
        expectedAmount: new anchor.BN(escrowAccountInfo.expectedAmount, 10, "le").toNumber()
    };
}
