// ============================================================
// EnergyLedger — Soroban Contract Architecture
// TypeScript interfaces + annotated Rust contract sketch
// ============================================================

// ── Shared types ──────────────────────────────────────────────

export type EnergySource = "solar" | "wind" | "hydro";

export interface EnergyToken {
  /** Token owner */
  owner: string;
  /** Amount in millWh (kWh × 1000 for precision) */
  amountMillWh: bigint;
  source: EnergySource;
  /** Unix timestamp of production */
  mintedAt: number;
  /** Set to true when retired/burned for carbon accounting */
  retired: boolean;
}

export interface MarketListing {
  id: string;
  seller: string;
  amountMillWh: bigint;
  /** Price in stroops (XLM × 10_000_000) per kWh */
  pricePerKWhStroops: bigint;
  source: EnergySource;
  location: string;
  createdAt: number;
  expiresAt: number;
  active: boolean;
}

export interface EscrowTrade {
  id: string;
  listingId: string;
  buyer: string;
  seller: string;
  amountMillWh: bigint;
  totalStroops: bigint;
  /** "pending" | "settled" | "cancelled" */
  status: "pending" | "settled" | "cancelled";
  createdAt: number;
  settledAt?: number;
}

// ── Contract method signatures ────────────────────────────────

/** energy_token contract */
export interface EnergyTokenContract {
  /**
   * Mint new kWh credits to `to` address.
   * Only callable by registered producers.
   * @param to  recipient address
   * @param amount  millWh amount
   */
  mint(to: string, amount: bigint): Promise<void>;

  /** Transfer credits between addresses */
  transfer(from: string, to: string, amount: bigint): Promise<void>;

  /** Read token balance in millWh */
  balance(address: string): Promise<bigint>;

  /**
   * Retire (burn) consumed credits permanently.
   * Emits a RetireEvent for carbon accounting.
   */
  retire(from: string, amount: bigint): Promise<void>;
}

/** marketplace contract */
export interface MarketplaceContract {
  /** Create a new listing; locks the seller's kWh tokens in escrow */
  createListing(
    seller: string,
    amount: bigint,
    pricePerKWhStroops: bigint,
    expiresIn: number
  ): Promise<string>; // returns listing ID

  /** Initiate buy — locks XLM in escrow, pending settlement */
  buyListing(listingId: string, buyer: string, amount: bigint): Promise<string>;

  /**
   * Settle escrow — atomically transfer:
   *   kWh tokens → buyer
   *   XLM        → seller (minus 0.5% protocol fee)
   */
  settle(tradeId: string): Promise<void>;

  /** Cancel an active listing and return locked tokens */
  cancelListing(listingId: string, seller: string): Promise<void>;

  /** Read a listing by ID */
  getListing(listingId: string): Promise<MarketListing>;
}
