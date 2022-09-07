import { Enum, Assignable, SCHEMA } from './solana-borsh';
import {
  AccountMeta,
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';

import BN from "bn.js";
import { deserialize } from 'borsh';

export class CreateAccount extends Assignable {
    programName: String;
    programIconCid: String;
    programDomainName: String;
    programRedirectUri: String[];
    bumpSeedNonce: number;
}

export class SetAuthority extends Assignable {}

export class CloseAccount extends Assignable {}

export class RelyingInstruction extends Enum {
    createAccount: CreateAccount;
    setAuthority: SetAuthority;
    closeAccount: CloseAccount;

    static createAccount(
      programName: String, 
      programIconCid: String, 
      programDomainName: String, 
      programRedirectUri: String[], 
      bumpSeedNonce: number
    ): RelyingInstruction {
        return new RelyingInstruction({ createAccount: new CreateAccount({
          programName,
          programIconCid,
          programDomainName,
          programRedirectUri,
          bumpSeedNonce,
        }) });
    }

    static setAuthority(): RelyingInstruction {
      return new RelyingInstruction({ setAuthority: new SetAuthority({}) });
    }

    static closeAccount(): RelyingInstruction {
      return new RelyingInstruction({ closeAccount: new CloseAccount({}) });
    }
}

export class RelatedProgramInfo extends Assignable {
    name: String;
    icon_cid: number[];
    domain_name: String;
    redirect_uri: String[];

    static default(): RelatedProgramInfo {
      return new RelatedProgramInfo({
        name: "",
        icon_cid: [],
        domain_name: "",
        redirect_uri: []
      });
    }
}

export class RelyingPartyData extends Assignable {
  version: number;
  authority: VPublicKey;
  related_program_data: RelatedProgramInfo;

  static default(): RelyingPartyData {
    return new RelyingPartyData({
      version: new BN(0),
      authority: VPublicKey.empty(),
      related_program_data: RelatedProgramInfo.default(),
    });
  }
}

// ///
// export function initialize(
//     solidAccount: PublicKey,
//     authority: PublicKey,
//     receiver: PublicKey
//   ): TransactionInstruction {
//     const keys: AccountMeta[] = [
//       // the DID account
//       { pubkey: solidAccount, isSigner: false, isWritable: true },
//       // a key with close permissions on the DID
//       { pubkey: authority, isSigner: true, isWritable: false },
//       // the account to receive the lamports
//       { pubkey: receiver, isSigner: false, isWritable: false },
//     ];
//     const data = RelyingInstruction.initialize().encode();
//     return new TransactionInstruction({
//       keys,
//       programId: new PublicKey("PROGRAM_ID"),
//       data,
//     });
// }

export class VPublicKey extends Assignable {
  // The public key bytes
  bytes: number[];

  toPublicKey(): PublicKey {
    return new PublicKey(this.bytes);
  }

  static parse(pubkey: string): VPublicKey {
    return VPublicKey.fromPublicKey(new PublicKey(pubkey));
  }

  static fromPublicKey(publicKey: PublicKey): VPublicKey {
    return new VPublicKey({ bytes: Uint8Array.from(publicKey.toBuffer()) });
  }

  static empty(): VPublicKey {
    const bytes = new Array(32);
    bytes.fill(0);
    return new VPublicKey({ bytes });
  }
}

SCHEMA.set(RelatedProgramInfo, {
  kind: 'struct',
  field: 'struct',
  fields: [
    ['name', 'String'],
    ['icon_cid', ['u8']],
    ['domain_name', 'String'],
    ['redirect_uri', ['String']],
  ],
});

SCHEMA.set(RelyingPartyData, {
  kind: 'struct',
  field: 'struct',
  fields: [
    ['version', 'u8'],
    ['authority', VPublicKey],
    ['related_program_data', RelatedProgramInfo],
  ],
});

SCHEMA.set(RelyingInstruction, {
    kind: 'enum',
    field: 'enum',
    values: [
      ['createAccount', CreateAccount],
      ['setAuthority', SetAuthority],
      ['closeAccount', CloseAccount],
    ],
});

SCHEMA.set(CreateAccount, { kind: 'struct', fields: [
    ['programName', 'String'],
    ['programIconCid', 'String'],
    ['programDomainName', 'String'],
    ['programRedirectUri', ['String']],
    ['bumpSeedNonce', 'u8'],
] });

SCHEMA.set(SetAuthority, { kind: 'struct', fields: [] });
SCHEMA.set(CloseAccount, { kind: 'struct', fields: [] });
