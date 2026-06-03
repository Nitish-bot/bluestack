import {
  Address,
  Keypair,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { readFile } from "node:fs/promises";
import { assert } from "chai";
import {
  QuasarSvm,
  createKeyedSystemAccount,
  type KeyedAccountInfo,
} from "@blueshift-gg/quasar-svm/web3.js";

const PROGRAM_ID = new Address("4ZmkkesWXMKvVKrrwxAz88sPYivrKevveg6pEPWmuDfW");
const LAMPORTS = 10_000_000_000n;

// quasar-svm wire format expects Address#toBuffer(); web3.js v3 exposes toBytes().
const proto = Address.prototype as Address & { toBuffer?: () => Buffer };
if (typeof proto.toBuffer !== "function") {
  proto.toBuffer = function (this: Address) {
    return Buffer.from(this.toBytes());
  };
}

function electionData(accounts: KeyedAccountInfo[], election: Address): Uint8Array {
  const raw = accounts.find((a) => a.accountId.equals(election))!.accountInfo.data;
  return raw instanceof Uint8Array ? raw : new Uint8Array(raw);
}

function parseVoteTotals(data: Uint8Array): number[] {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const candidatesLen = view.getUint16(66, true);
  const votesLen = view.getUint16(68, true);
  const votesOffset = 70 + candidatesLen * 32;
  return Array.from({ length: votesLen }, (_, i) =>
    view.getUint32(votesOffset + i * 4, true),
  );
}

function parseWinner(data: Uint8Array): Address | null {
  const winnerTag = data[33];
  return winnerTag === 1 ? new Address(data.slice(34, 66)) : null;
}

describe("bluestack", () => {
  let vm: QuasarSvm;
  let payer: Address;
  let election: Address;
  let candidates: Address[];
  let accounts: KeyedAccountInfo[];

  before(async () => {
    vm = new QuasarSvm({ token: false, token2022: false, associatedToken: false });
    vm.addProgram(PROGRAM_ID, await readFile("target/deploy/bluestack.so"));

    payer = (await Keypair.generate()).address;
    [election] = await Address.findProgramAddress(
      [Buffer.from("election"), Buffer.from(payer.toBytes())],
      PROGRAM_ID,
    );
    candidates = await Promise.all(
      [0, 1, 2].map(async () => (await Keypair.generate()).address),
    );
  });

  it("create election", () => {
    const createData = Buffer.alloc(1 + 2 + 96);
    createData.writeUInt8(0, 0);
    createData.writeUInt16LE(3, 1);
    candidates.forEach((c, i) => createData.set(c.toBytes(), 3 + i * 32));

    accounts = vm.processInstruction(
      new TransactionInstruction({
        programId: PROGRAM_ID,
        keys: [
          { pubkey: payer, isSigner: true, isWritable: true },
          { pubkey: election, isSigner: false, isWritable: true },
          { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: createData,
      }),
      [
        createKeyedSystemAccount(payer, LAMPORTS),
        {
          accountId: election,
          accountInfo: {
            lamports: 0n,
            data: Buffer.alloc(0),
            owner: SystemProgram.programId,
            executable: false,
            rentEpoch: 0n,
            space: 0n,
          },
        },
      ],
    ).accounts;

    const data = electionData(accounts, election);
    assert.equal(data[0], 1, "election discriminator");
    assert.isTrue(new Address(data.slice(1, 33)).equals(payer));
    assert.isNull(parseWinner(data));
    assert.deepEqual(parseVoteTotals(data), [0, 0, 0]);
  });

  it("vote", () => {
    const counts = [3, 1, 2] as const;
    for (let i = 0; i < 3; i++) {
      for (let n = 0; n < counts[i]; n++) {
        const voteData = Buffer.alloc(33);
        voteData.writeUInt8(1, 0);
        voteData.set(candidates[i].toBytes(), 1);
        accounts = vm.processInstruction(
          new TransactionInstruction({
            programId: PROGRAM_ID,
            keys: [
              { pubkey: payer, isSigner: true, isWritable: true },
              { pubkey: election, isSigner: false, isWritable: true },
            ],
            data: voteData,
          }),
          accounts,
        ).accounts;
      }
    }

    assert.deepEqual(parseVoteTotals(electionData(accounts, election)), [3, 1, 2]);
    assert.isNull(parseWinner(electionData(accounts, election)));
  });

  it("declare winner", () => {
    accounts = vm.processInstruction(
      new TransactionInstruction({
        programId: PROGRAM_ID,
        keys: [
          { pubkey: payer, isSigner: true, isWritable: false },
          { pubkey: election, isSigner: false, isWritable: true },
        ],
        data: Buffer.from([2]),
      }),
      accounts,
    ).accounts;

    const data = electionData(accounts, election);
    assert.isTrue(parseWinner(data)!.equals(candidates[0]));
    assert.deepEqual(parseVoteTotals(data), [3, 1, 2]);
  });
});
