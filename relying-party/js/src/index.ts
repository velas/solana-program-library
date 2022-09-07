import {
  Account,
  AccountMeta,
  Connection,
  TransactionInstruction,
  Transaction,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import { createHash } from 'crypto';

import { Assignable } from './solana-borsh'
import { RelyingPartyData, RelyingInstruction } from './instruction'

const RELYING_CODE_ADDRESS = new PublicKey("VRPLtk4k31bDL99mn1A5mE96CUUzQ9PnftEwf2LvMiG");


function getRedirectUriHash(programRedirectUri: String[]): Buffer {
  createHash('sha256');
  let hasher = createHash('sha256');
  for (let uri of programRedirectUri) {
    hasher.update(uri.toString());
  }

  return hasher.digest();
}

/**
* Generate RelyingParty address.
* 
* @param owner genesis owner of the RelyingParty
* @returns RelyingParty Public Key
*/
async function generateAccountAddress(
    programName: String, 
    programIconCid: String, 
    programDomainName: String, 
    programRedirectUri: String[],
  ): Promise<[PublicKey, number]> {
    const redirect_uri_hash = getRedirectUriHash(programRedirectUri);

    const relying = await PublicKey.findProgramAddress(
        [
          Buffer.from(programName.slice(0, Math.min(32, programName.length))),
          Buffer.from(programIconCid.slice(0, Math.min(32, programName.length))),
          Buffer.from(programDomainName.slice(0, Math.min(32, programName.length))),
          redirect_uri_hash.slice(0, Math.min(32, redirect_uri_hash.length))
        ],
        RELYING_CODE_ADDRESS
      );

    return relying
}

/**
* Generate signers from accounts.
* 
* @param accounts accounts that will become signers
* @returns Signers
*/
function accountsToSigners(accounts: Account[]): Array<AccountMeta> {
  const signers = new Array<AccountMeta>();
  for (const account of accounts) {
      signers.push({pubkey: account.publicKey, isSigner: true, isWritable: false})
  }

  return signers;
}

/**
* Get parsed RelyingParty.
* 
* @param connection connection to the node
* @param relyingPartyAddress Public Key
* @returns Parsed RelyingPartyData struct
*/
async function getRelyingParty(connection: Connection, relyingPartyAddress: PublicKey): Promise<Assignable> {
  const relyingInfo = await connection.getAccountInfo(relyingPartyAddress);

  return RelyingPartyData.decode(relyingInfo?.data as any);
}

/**
* Get parsed RelyingParty unckecked.
* 
* @param connection connection to the node
* @param relyingPartyAddress Public Key
* @returns Parsed struct
*/
async function getRelyingPartyUnchecked(connection: Connection, relyingPartyAddress: PublicKey): Promise<Assignable> {
  const relyingInfo = await connection.getAccountInfo(relyingPartyAddress);

  if (!relyingInfo || !relyingInfo.data.length){
      return RelyingPartyData.default();
  }

  return RelyingPartyData.decode(relyingInfo.data as any);
}

/**
* Create Account to RelyingParty instruction. 
* 
* @param owner new owner of the  RelyingParty
* @returns `SetAuthority` TransactionInstruction 
*/
async function createAccount(
    owner: PublicKey,
    programName: String, 
    programIconCid: String, 
    programDomainName: String, 
    programRedirectUri: String[], 
  ): Promise<TransactionInstruction> {
  const relyingPartyAddress = await generateAccountAddress(programName, programIconCid, programDomainName, programRedirectUri);
  const keys = [
      {pubkey: relyingPartyAddress[0], isSigner: false, isWritable: true},
      {pubkey: owner, isSigner: true, isWritable: false},
      {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
      {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
    ];

  return new TransactionInstruction({
      programId: RELYING_CODE_ADDRESS,
      keys: keys,
      data: RelyingInstruction.createAccount(
        programName,
        programIconCid,
        programDomainName,
        programRedirectUri,
        relyingPartyAddress[1],
      ).encode(),
  });
}

/**
* Set Authority to RelyingParty instruction. 
* 
* @param owner new owner of the  RelyingParty
* @returns `SetAuthority` TransactionInstruction 
*/
async function setAuthority(relyingPartyAddress: PublicKey, owner: PublicKey, newAuthority: PublicKey): Promise<TransactionInstruction> {
  const keys = [
      {pubkey: relyingPartyAddress, isSigner: false, isWritable: true},
      {pubkey: owner, isSigner: true, isWritable: false},
      {pubkey: newAuthority, isSigner: false, isWritable: false},

      // {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
      // {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
    ];

  return new TransactionInstruction({
      programId: RELYING_CODE_ADDRESS,
      keys: keys,
      data: RelyingInstruction.setAuthority().encode(),
  });
}

/**
* Close Account to RelyingParty instruction. 
* 
* @param owner owner of the  RelyingParty
* @returns `CloseAccount` TransactionInstruction 
*/
async function closeAccount(relyingPartyAddress: PublicKey, owner: PublicKey, destination: PublicKey): Promise<TransactionInstruction> {
  const keys = [
      {pubkey: relyingPartyAddress, isSigner: false, isWritable: true},
      {pubkey: owner, isSigner: true, isWritable: false},
      {pubkey: destination, isSigner: false, isWritable: false},
    ];

  return new TransactionInstruction({
      programId: RELYING_CODE_ADDRESS,
      keys: keys,
      data: RelyingInstruction.closeAccount().encode(),
  });
}

async function main() {
  const txIx =  createAccount(
    new PublicKey(22),
    "test_application_client",
    "QmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "domain.name",
    ["red1", "red2"]
  );
  console.log(txIx);
}


void main()
.then(err => {
  console.error(err)
})
.then(() => process.exit())