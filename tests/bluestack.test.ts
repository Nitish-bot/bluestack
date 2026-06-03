import { Keypair } from "@solana/web3.js";
import { BluestackClient } from "../target/client/typescript/bluestack/web3.js";
import { readFile } from "node:fs/promises";
import { describe, it, run } from "mocha";
import { assert } from "chai";
import { QuasarSvm, createKeyedSystemAccount } from "@blueshift-gg/quasar-svm/dist/kit";

const BluestackProgram = new BluestackClient();

describe("Bluestack Program", async () => {
  const vm = new QuasarSvm();
  // @ts-ignore programId does have address type
  vm.addProgram(BluestackClient.programId, await readFile("target/deploy/bluestack.so"));

  const { address: payer } = await Keypair.generate();

  it("initializes", async () => {
    const initializeInstruction = BluestackProgram.createInitializeInstruction({
      payer,
    });

    const result = vm.processInstruction(initializeInstruction, [
      createKeyedSystemAccount(payer),
    ]);

    assert.isTrue(result.status.ok, `initialize failed:\n${result.logs.join("\n")}`);
  });

  run();
});
