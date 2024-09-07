import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Fame } from "../app/src/idl/fame";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {
    createMint,
    mintTo,
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
    getOrCreateAssociatedTokenAccount,
    transfer,
    ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

chai.use(chaiAsPromised);

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const PROGRAM_SEED = Buffer.from("program_state");

declare global {
    namespace Chai {
        interface Assertion {
            rejectedWithLog(error: string): Promise<void>;
            rejectedWithAnchorError(
                idl: anchor.Idl,
                errorName: string,
                programId: PublicKey,
            ): Promise<void>;
        }
    }
}

const expectAnchorError =
    (errorCode: number, errorName: string, programId?: PublicKey) =>
    (anchorErr: any) => {
        const normalisedError = anchor.AnchorError.parse(anchorErr.logs);
        function convertFirstToLower(input: string) {
            if (!input) return "";
            return input.charAt(0).toLowerCase() + input.slice(1);
        }
        expect(normalisedError).not.to.be.null;
        let actualErrorCode = convertFirstToLower(
            normalisedError!.error.errorCode.code,
        );
        expect(actualErrorCode).to.equal(errorName);
        expect(normalisedError!.error.errorCode.number).to.equal(errorCode);
        if (programId) {
            expect(normalisedError!.program.equals(programId)).is.true;
        }
    };

chai.Assertion.addMethod("rejectedWithLog", function (error: string) {
    return expect(this._obj).to.be.rejected.then((err) => {
        let logs = err.logs as string[];
        let found = logs.find((log: string) => log.includes(error));
        expect(found).to.be.contains(error);
    });
});

chai.Assertion.addMethod(
    "rejectedWithAnchorError",
    function (idl: anchor.Idl, name: string, programId?: PublicKey) {
        const found = idl.errors?.find((e) => e.name === name);
        if (!found) throw new Error(`No error with name ${name} found in IDL`);
        return expect(this._obj).to.be.rejected.then(
            expectAnchorError(found.code, name, programId),
        );
    },
);

class Token {
    mint: Keypair;
    owner: Keypair;
    ownerATA: PublicKey;
    program: anchor.Program<Fame>;
    constructor(args: {
        mint: Keypair;
        owner: Keypair;
        ownerATA: PublicKey;
        program: anchor.Program<Fame>;
    }) {
        this.mint = args.mint;
        this.owner = args.owner;
        this.ownerATA = args.ownerATA;
        this.program = args.program;
    }

    static async createToken(
        program: anchor.Program<Fame>,
        owner: Keypair,
        name: string,
        symbol: string,
    ): Promise<Token> {
        console.log("Create token for " + name + "/" + symbol);
        const [programPDA] = anchor.web3.PublicKey.findProgramAddressSync(
            [PROGRAM_SEED],
            program.programId,
        );

        const mintKp = new anchor.web3.Keypair();
        const mint = await createMint(
            program.provider.connection,
            owner,
            owner.publicKey,
            null,
            9,
            mintKp,
            { commitment: "confirmed" },
            TOKEN_PROGRAM_ID,
        );
        console.log("Token created: " + mintKp.publicKey.toBase58());

        const programATA = await getOrCreateAssociatedTokenAccount(
            program.provider.connection,
            owner,
            mint,
            programPDA,
            true,
            undefined,
            { commitment: "confirmed" },
        );

        const ownerATA = await getOrCreateAssociatedTokenAccount(
            program.provider.connection,
            owner,
            mint,
            owner.publicKey,
            true,
            undefined,
            { commitment: "confirmed" },
        );

        return new Token({
            owner,
            mint: mintKp,
            ownerATA: ownerATA.address,
            program,
        });
    }

    async mintTo(args: {
        user: Keypair;
        amount: number;
    }): Promise<anchor.web3.TransactionSignature> {
        const userATA = await getOrCreateAssociatedTokenAccount(
            this.program.provider.connection,
            args.user,
            this.mint.publicKey,
            args.user.publicKey,
            true,
            undefined,
            { commitment: "confirmed" },
        );

        console.log(
            `Mint ${args.amount} tokens to ${userATA.address.toBase58()}`,
        );

        return await mintTo(
            this.program.provider.connection,
            this.owner,
            this.mint.publicKey,
            userATA.address,
            this.owner.publicKey,
            args.amount,
            [],
            { commitment: "confirmed" },
            TOKEN_PROGRAM_ID,
        );
    }

    async transferTo(args: {
        sender: Keypair;
        receiver: PublicKey;
        amount: number;
    }): Promise<anchor.web3.TransactionSignature> {
        console.log("Transfer tokens to " + args.receiver.toBase58());
        const senderATA = await getOrCreateAssociatedTokenAccount(
            this.program.provider.connection,
            args.sender,
            this.mint.publicKey,
            args.sender.publicKey,
            true,
            undefined,
            { commitment: "confirmed" },
        );
        const receiverATA = await getOrCreateAssociatedTokenAccount(
            this.program.provider.connection,
            args.sender,
            this.mint.publicKey,
            args.receiver,
            true,
            undefined,
            { commitment: "confirmed" },
        );

        return await transfer(
            this.program.provider.connection,
            args.sender,
            senderATA.address,
            receiverATA.address,
            args.sender,
            args.amount,
            [],
            { commitment: "confirmed" },
            TOKEN_PROGRAM_ID,
        );
    }

    getAccountFor(user: Keypair | PublicKey) {
        if (user instanceof Keypair) {
            user = user.publicKey;
        }
        return getAssociatedTokenAddressSync(this.mint.publicKey, user, true);
    }

    async getBalanceIntFor(user: Keypair | PublicKey) {
        let balance = await this.getBalanceFor(user);
        return Math.floor(
            balance.value.uiAmount * 10 ** balance.value.decimals,
        );
    }

    async getBalanceFor(user: Keypair | PublicKey) {
        if (user instanceof Keypair) {
            user = user.publicKey;
        }
        return await this.program.provider.connection.getTokenAccountBalance(
            this.getAccountFor(user),
        );
    }
}

describe("fame", () => {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);
    const owner = Keypair.generate();
    const user1 = Keypair.generate();
    const user2 = Keypair.generate();
    const user3 = Keypair.generate();
    let token1: Token = null;

    let token2: Token = null;

    const program = anchor.workspace.Fame as Program<Fame>;

    type Event = anchor.IdlEvents<(typeof program)["idl"]>;
    const getEvent = async <E extends keyof Event>(
        eventName: E,
        fn: Promise<anchor.web3.TransactionResponse>,
    ) => {
        let listenerId: number;
        const event = await new Promise<Event[E]>(async (res) => {
            let timeout = setTimeout(() => {
                program.removeEventListener(listenerId);
                throw new Error(`Timeout waiting for event ${eventName}`);
            }, 4000);
            listenerId = program.addEventListener(eventName, async (event) => {
                clearTimeout(timeout);
                res(event);
            });
            await fn;
            await program.removeEventListener(listenerId);
        });
        return event;
    };

    const airdrop = async (user: Keypair, lamports: number) => {
        console.log("Airdrop " + user.publicKey.toBase58() + " " + lamports);
        const airdropSignature = await provider.connection.requestAirdrop(
            user.publicKey,
            lamports,
        );

        await provider.connection.confirmTransaction(airdropSignature);
    };

    const [programPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [PROGRAM_SEED],
        program.programId,
    );

    const initialize = async (args: { admin: Keypair }) => {
        console.log(
            "Initialize " +
                args.admin.publicKey.toBase58() +
                " in " +
                programPDA.toBase58(),
        );
        console.log("State: " + programPDA.toBase58());

        const accounts = {
            signer: args.admin.publicKey,
            state: programPDA,
            systemProgram: anchor.web3.SystemProgram.programId,
        };

        const builder = program.methods
            .initialize({ admin: args.admin.publicKey })
            .accounts(accounts)
            .signers([args.admin]);
        try {
            const signature = await builder.rpc({ commitment: "confirmed" });
            const receipt = await provider.connection.getTransaction(
                signature,
                {
                    commitment: "confirmed",
                },
            );
            return receipt;
        } catch (e) {
            console.log("Error on initialize: ", e);
            throw e;
        }
    };

    const createToken = async (args: {
        admin: Keypair;
        name: string;
        symbol: string;
        socialAccountUrl: string;
    }) => {
        console.log(`Create token ${args.name} (${args.symbol})`);
        const accounts = {
            creator: args.admin.publicKey,
            mint: token1.mint.publicKey,
            creatorTokenAccount: token1.getAccountFor(args.admin.publicKey),
            tokenInfo: anchor.web3.PublicKey.findProgramAddressSync(
                [Buffer.from("token_info"), token1.mint.publicKey.toBuffer()],
                program.programId,
            )[0],
            bondingCurve: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("bonding_curve"),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            liquidityPool: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("liquidity_pool"),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        };

        const builder = program.methods
            .createToken(args.name, args.symbol, args.socialAccountUrl)
            .accounts(accounts)
            .signers([args.admin]);

        try {
            const signature = await builder.rpc({ commitment: "confirmed" });
            const receipt = await provider.connection.getTransaction(
                signature,
                {
                    commitment: "confirmed",
                },
            );
            return receipt;
        } catch (e) {
            console.log("Error on create token: ", e);
            throw e;
        }
    };

    const mintToken = async (args: { user: Keypair; amount: anchor.BN }) => {
        console.log(
            `Mint ${args.amount.toString()} tokens to ${args.user.publicKey.toBase58()}`,
        );
        const accounts = {
            user: args.user.publicKey,
            tokenInfo: anchor.web3.PublicKey.findProgramAddressSync(
                [Buffer.from("token_info"), token1.mint.publicKey.toBuffer()],
                program.programId,
            )[0],
            bondingCurve: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("bonding_curve"),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            liquidityPool: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("liquidity_pool"),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            userPortfolio: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("user_portfolio"),
                    args.user.publicKey.toBuffer(),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            mint: token1.mint.publicKey,
            userTokenAccount: token1.getAccountFor(args.user.publicKey),
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
        };
        const builder = program.methods
            .mintToken(args.amount)
            .accounts(accounts)
            .signers([args.user]);

        try {
            const signature = await builder.rpc({ commitment: "confirmed" });
            const receipt = await provider.connection.getTransaction(
                signature,
                {
                    commitment: "confirmed",
                },
            );
            return receipt;
        } catch (e) {
            console.log("Error on mint token: ", e);
            throw e;
        }
    };

    const burnToken = async (args: { user: Keypair; amount: anchor.BN }) => {
        console.log(
            `Burn ${args.amount.toString()} tokens from ${args.user.publicKey.toBase58()}`,
        );
        const accounts = {
            user: args.user.publicKey,
            tokenInfo: anchor.web3.PublicKey.findProgramAddressSync(
                [Buffer.from("token_info"), token1.mint.publicKey.toBuffer()],
                program.programId,
            )[0],
            bondingCurve: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("bonding_curve"),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            liquidityPool: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("liquidity_pool"),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            userPortfolio: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("user_portfolio"),
                    args.user.publicKey.toBuffer(),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            mint: token1.mint.publicKey,
            userTokenAccount: token1.getAccountFor(args.user.publicKey),
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
        };
        const builder = program.methods
            .burnToken(args.amount)
            .accounts(accounts)
            .signers([args.user]);

        try {
            const signature = await builder.rpc({ commitment: "confirmed" });
            const receipt = await provider.connection.getTransaction(
                signature,
                {
                    commitment: "confirmed",
                },
            );
            return receipt;
        } catch (e) {
            console.log("Error on burn token: ", e);
            throw e;
        }
    };

    const withdrawFees = async (args: {
        admin: Keypair;
        amount: anchor.BN;
    }) => {
        console.log(`Withdraw ${args.amount.toString()} fees`);
        const accounts = {
            admin: args.admin.publicKey,
            liquidityPool: anchor.web3.PublicKey.findProgramAddressSync(
                [
                    Buffer.from("liquidity_pool"),
                    token1.mint.publicKey.toBuffer(),
                ],
                program.programId,
            )[0],
            feeReceiver: args.admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        };
        const builder = program.methods
            .withdrawFees(args.amount)
            .accounts(accounts)
            .signers([args.admin]);

        try {
            const signature = await builder.rpc({ commitment: "confirmed" });
            const receipt = await provider.connection.getTransaction(
                signature,
                {
                    commitment: "confirmed",
                },
            );
            return receipt;
        } catch (e) {
            console.log("Error on withdraw fees: ", e);
            throw e;
        }
    };

    before(async () => {
        await Promise.all([
            airdrop(owner, 10 * anchor.web3.LAMPORTS_PER_SOL),
            airdrop(user1, 10 * anchor.web3.LAMPORTS_PER_SOL),
            airdrop(user2, 10 * anchor.web3.LAMPORTS_PER_SOL),
            airdrop(user3, 10 * anchor.web3.LAMPORTS_PER_SOL),
        ]);
        [token1, token2] = await Promise.all([
            Token.createToken(program, owner, "Token #1", "TST1"),
            Token.createToken(program, owner, "Token #2", "TST2"),
        ]);
        await token1.mintTo({ user: owner, amount: 1024_000_000_000 });
    });

    it("Is initialized!", async () => {
        const receipt = await initialize({ admin: owner });
        expect(receipt.meta.err).to.be.null;
    });

    it("Impossible to init twice", async () => {
        await expect(initialize({ admin: owner })).to.be.rejectedWithLog(
            "already in use",
        );
    });

    it("Create a valid token", async () => {
        let event = await getEvent(
            "tokenCreated",
            createToken({
                admin: owner,
                name: "Test Token",
                symbol: "TEST",
                socialAccountUrl: "https://example.com",
            }),
        );
        expect(event.token.toBase58()).to.equal(
            token1.mint.publicKey.toBase58(),
        );
        expect(event.name).to.equal("Test Token");
        expect(event.symbol).to.equal("TEST");
        expect(event.socialAccountUrl).to.equal("https://example.com");
        expect(event.creator.toBase58()).to.equal(owner.publicKey.toBase58());
    });

    it("Mint tokens", async () => {
        const amountToMint = new anchor.BN(100_000_000); // 100 tokens
        let event = await getEvent(
            "tokenMinted",
            mintToken({
                user: user1,
                amount: amountToMint,
            }),
        );
        expect(event.token.toBase58()).to.equal(
            token1.mint.publicKey.toBase58(),
        );
        expect(event.user.toBase58()).to.equal(user1.publicKey.toBase58());
        expect(event.amount.eq(amountToMint)).to.be.true;
    });

    it("Burn tokens", async () => {
        const amountToBurn = new anchor.BN(50_000_000); // 50 tokens
        let event = await getEvent(
            "tokenBurned",
            burnToken({
                user: user1,
                amount: amountToBurn,
            }),
        );
        expect(event.token.toBase58()).to.equal(
            token1.mint.publicKey.toBase58(),
        );
        expect(event.user.toBase58()).to.equal(user1.publicKey.toBase58());
        expect(event.amount.eq(amountToBurn)).to.be.true;
    });

    it("Withdraw fees", async () => {
        const amountToWithdraw = new anchor.BN(1_000_000); // 1 token worth of fees
        let event = await getEvent(
            "feeWithdrawn",
            withdrawFees({
                admin: owner,
                amount: amountToWithdraw,
            }),
        );
        expect(event.amount.eq(amountToWithdraw)).to.be.true;
        expect(event.receiver.toBase58()).to.equal(owner.publicKey.toBase58());
    });

    it("Cannot withdraw fees if not admin", async () => {
        const amountToWithdraw = new anchor.BN(1_000_000);
        await expect(
            withdrawFees({
                admin: user1, // user1 is not the admin
                amount: amountToWithdraw,
            }),
        ).to.be.rejectedWithAnchorError(
            program.idl,
            "Unauthorized",
            program.programId,
        );
    });

    it("Cannot mint more tokens than allowed by bonding curve", async () => {
        const tooManyTokens = new anchor.BN(1_000_000_000_000); // Very large number of tokens
        await expect(
            mintToken({
                user: user2,
                amount: tooManyTokens,
            }),
        ).to.be.rejectedWithAnchorError(
            program.idl,
            "InsufficientLiquidity",
            program.programId,
        );
    });

    it("Cannot burn more tokens than user owns", async () => {
        const tooManyTokens = new anchor.BN(1_000_000_000_000); // Very large number of tokens
        await expect(
            burnToken({
                user: user2,
                amount: tooManyTokens,
            }),
        ).to.be.rejectedWithAnchorError(
            program.idl,
            "InsufficientBalance",
            program.programId,
        );
    });
});
