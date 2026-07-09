# EnergyLedger Smart Contracts Documentation Index

Complete guide to the EnergyLedger Soroban smart contracts for renewable energy trading.

## 📚 Documentation Map

### Getting Started
1. **[QUICKSTART.md](./QUICKSTART.md)** - 10-minute deployment guide
   - Build contracts
   - Deploy to Testnet
   - Run end-to-end test flow
   - Store contract IDs for later use

2. **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** - Project overview
   - What was delivered
   - Architecture decisions
   - Security features
   - File structure

### Detailed Reference
3. **[CONTRACTS.md](./CONTRACTS.md)** - Complete technical documentation
   - Contract architecture
   - Full deployment guide
   - Usage examples for all functions
   - Testing instructions
   - Security considerations
   - Units and precision
   - Events reference

4. **[CONTRACT_API.md](./CONTRACT_API.md)** - Complete API reference
   - All contract functions with detailed specs
   - Parameter descriptions
   - Return values
   - Authorization requirements
   - Event definitions
   - Error codes
   - Gas estimates

### Implementation
5. **[src/contracts/energy_token.rs](./src/contracts/energy_token.rs)** - Energy Token contract
   - Full Rust implementation (~250 lines)
   - Complete test suite
   - Event emissions

6. **[src/contracts/marketplace.rs](./src/contracts/marketplace.rs)** - Marketplace contract
   - Full Rust implementation (~350 lines)
   - Atomic settlement logic
   - Escrow tracking
   - Complete test suite

7. **[src/lib/soroban-integration.ts](./src/lib/soroban-integration.ts)** - Frontend integration
   - TypeScript wrapper classes
   - Contract interfaces
   - Unit conversion utilities
   - Event parsing

---

## 🎯 Quick Reference

### I want to...

**...understand the system**
→ Start with [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)

**...deploy to testnet quickly**
→ Follow [QUICKSTART.md](./QUICKSTART.md)

**...understand contract details**
→ Read [CONTRACTS.md](./CONTRACTS.md)

**...call a specific contract function**
→ Look it up in [CONTRACT_API.md](./CONTRACT_API.md)

**...see the Rust code**
→ Open [energy_token.rs](./src/contracts/energy_token.rs) or [marketplace.rs](./src/contracts/marketplace.rs)

**...integrate with frontend**
→ Use utilities in [soroban-integration.ts](./src/lib/soroban-integration.ts)

**...understand security**
→ See "Security Considerations" in [CONTRACTS.md](./CONTRACTS.md)

**...see usage examples**
→ Check [CONTRACTS.md](./CONTRACTS.md) and [CONTRACT_API.md](./CONTRACT_API.md) examples

**...understand events**
→ See "Events" section in [CONTRACTS.md](./CONTRACTS.md) and [CONTRACT_API.md](./CONTRACT_API.md)

---

## 📋 Two-Contract Architecture

```
┌─────────────────────────────────────────────────────────┐
│           EnergyLedger Marketplace                       │
└─────────────────────────────────────────────────────────┘
                          │
                 ┌────────┴────────┐
                 ↓                 ↓
        ┌──────────────┐   ┌──────────────┐
        │ Energy Token │   │ Marketplace  │
        │  Contract    │   │  /Escrow     │
        └──────────────┘   │  Contract    │
                           └──────────────┘
                                 │
                          ┌──────┴────────┐
                          ↓               ↓
                       Seller          Buyer
                      (Producer)      (Consumer)
```

### Energy Token Contract
- Tokenizes renewable energy as kWh/millWh
- Manages producer registration
- Tracks balances
- Enables retirement (burning) for carbon accounting

### Marketplace Contract
- Creates energy trading listings
- Locks tokens in escrow
- Matches buyers and sellers
- Executes atomic settlement with fee deduction
- Prevents counterparty risk

---

## 🚀 Deployment Checklist

- [ ] Install Rust 1.70+ with wasm32 target
- [ ] Install Soroban CLI
- [ ] Create Stellar testnet account
- [ ] Get testnet XLM from friendbot
- [ ] Build contracts: `soroban contract build`
- [ ] Deploy energy token contract
- [ ] Initialize energy token contract
- [ ] Deploy marketplace contract
- [ ] Initialize marketplace contract
- [ ] Register a test producer
- [ ] Mint test energy tokens
- [ ] Create test listing
- [ ] Execute end-to-end test trade
- [ ] Store contract IDs in `.env.contracts`

See [QUICKSTART.md](./QUICKSTART.md) for step-by-step commands.

---

## 📊 Contract Statistics

### Energy Token Contract
- Lines of code: ~250
- Functions: 7 public, 1 test
- Storage keys: 4
- Events: 4
- Test coverage: 4 test cases
- Dependencies: soroban-sdk v21.0

### Marketplace Contract
- Lines of code: ~350
- Functions: 8 public, 2 test
- Storage keys: 7
- Events: 4
- Test coverage: 3 test cases
- Dependencies: soroban-sdk v21.0

### Total
- Combined LOC: ~600
- Total functions: 15 public
- Total test cases: 7
- Build output: ~50 KB WASM per contract

---

## 🔐 Security Features

| Feature | Implementation |
|---------|-----------------|
| Authorization | `require_auth()` on all critical operations |
| Escrow | Tokens locked until settlement |
| Atomic swaps | Both sides complete or both revert |
| Balance verification | Checked before every transfer |
| Input validation | All amounts must be positive |
| Expiration limits | Listings expire after specified ledger |
| Fee protection | Hard-coded 0.5% fee, immutable treasury |
| Event logging | All state changes emitted as events |

---

## 💡 Key Concepts

### millWh (Milliwatt-Hours)
- Precision unit for energy trading
- 1 kWh = 1,000 millWh
- Enables granular P2P trading
- Stored as i128

### stroops (Stellar Lumens)
- Base unit for XLM on Stellar
- 1 XLM = 10,000,000 stroops
- Used for payment atomicity
- Stored as i128

### Escrow Pattern
- Seller locks energy tokens
- Buyer locks XLM
- Settlement swaps both atomically
- Prevents counterparty risk

### Producer Registry
- Only registered producers can mint
- Prevents token inflation
- Admin-controlled authorization
- Enables identity verification

### Protocol Fees
- 0.5% (50 basis points) on every trade
- Auto-deducted at settlement
- Routed to treasury
- Funds ecosystem development

---

## 📞 Support

### Documentation
- **API Reference**: [CONTRACT_API.md](./CONTRACT_API.md)
- **Technical Details**: [CONTRACTS.md](./CONTRACTS.md)
- **Quick Start**: [QUICKSTART.md](./QUICKSTART.md)

### External Resources
- **Soroban Docs**: https://soroban.stellar.org
- **Stellar Docs**: https://developers.stellar.org
- **GitHub Repo**: https://github.com/Stellar-Energy-Ledger/EnergyLedger

### Testing
- Run `cargo test` in contract directories
- Check test cases in contract source files
- See QUICKSTART.md for end-to-end flow

---

## 🔄 Development Workflow

1. **Local Development**
   ```bash
   cargo test  # Run local tests
   ```

2. **Build for Deployment**
   ```bash
   soroban contract build
   ```

3. **Deploy to Testnet**
   ```bash
   soroban contract deploy --wasm <wasm-file>
   ```

4. **Initialize Contracts**
   ```bash
   soroban contract invoke --id <contract-id> -- initialize ...
   ```

5. **Interact with Contracts**
   ```bash
   soroban contract invoke --id <contract-id> -- <function> ...
   ```

6. **Query State**
   ```bash
   soroban contract invoke --id <contract-id> -- <query-function>
   ```

---

## 📈 Future Enhancements

Planned improvements documented in [CONTRACTS.md](./CONTRACTS.md):

- Multi-signature admin controls
- Dynamic fee adjustments
- Batch operations
- Cross-contract composability
- Oracle integration
- Governance tokens
- Advanced escrow mechanisms
- Historical data archival

---

## 📄 File Structure

```
EnergyLedger/
├── CONTRACTS_INDEX.md              ← You are here
├── QUICKSTART.md                   ← Start here
├── CONTRACTS.md                    ← Full documentation
├── CONTRACT_API.md                 ← API reference
├── IMPLEMENTATION_SUMMARY.md       ← Project overview
├── Cargo.toml                      ← Workspace config
├── src/
│   ├── contracts/
│   │   ├── energy_token.rs        ← Energy token implementation
│   │   ├── energy_token_Cargo.toml
│   │   ├── marketplace.rs         ← Marketplace implementation
│   │   └── marketplace_Cargo.toml
│   └── lib/
│       └── soroban-integration.ts  ← TypeScript integration
└── [Other project files...]
```

---

## ✅ Validation Checklist

- [x] Energy token contract compiles
- [x] Marketplace contract compiles
- [x] All tests pass locally
- [x] Authorization checks implemented
- [x] Events emitted correctly
- [x] Error handling robust
- [x] Documentation complete
- [x] API reference comprehensive
- [x] Quick start guide tested
- [x] TypeScript integration stubs ready

---

## 🎓 Learning Path

**Beginner**: Follow the Quick Start → Understand the flow → Read API docs
**Intermediate**: Study contract code → Understand security → Deploy testnet
**Advanced**: Customize contracts → Add features → Deploy mainnet

---

## 📝 License

EnergyLedger smart contracts are part of the EnergyLedger project. See repository for license information.

---

## 🤝 Contributing

To contribute improvements to the contracts:

1. Read [CONTRACTS.md](./CONTRACTS.md) for architecture
2. Understand security model (see "Security Considerations")
3. Make changes with full test coverage
4. Run `cargo test` to verify
5. Submit PR with documentation updates

---

**Version**: 1.0.0  
**Last Updated**: July 2026  
**Status**: Production-Ready  
**Soroban SDK**: v21.0

Start with [QUICKSTART.md](./QUICKSTART.md) to get running in 10 minutes! 🚀
