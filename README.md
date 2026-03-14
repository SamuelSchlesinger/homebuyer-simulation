# Homebuyer Monte Carlo Simulation

A desktop app that helps first-time home buyers and anyone assessing a house compare **buying vs renting** under uncertainty. It runs thousands of scenarios to show the range of possible outcomes, not just a single point estimate.

## What This App Tells You

- **Effective monthly cost** of buying (median and range).
- **Final equity** distribution at the end of your holding period.
- **Probability buying is cheaper** than renting.
- **Risk indicators**, like probability of negative equity.

## How It Works (Plain English)

The app simulates many possible futures for home prices and costs. Each simulation:

1. Computes mortgage payments from your inputs.
2. Updates home value using a sampled appreciation rate.
3. Applies property tax and insurance increases.
4. Adds monthly costs: mortgage, taxes, insurance, repairs, HOA, PMI.
5. Computes **equity if you sold at that month**.
6. Converts everything into **effective monthly cost**.

## Key Definitions

- **Equity** = home value - remaining loan balance - selling costs
- **Cumulative cost** = down payment + buying costs + monthly out-of-pocket costs
- **Effective cost** = cumulative cost - equity
- **Effective monthly cost** = effective cost / months

These formulas **include** the buying and selling costs you set.

## Getting Started

### Requirements

- Rust (stable). Install via rustup.

### Run the App

```bash
cargo run --release
```

## Using the App

1. **Setup tab**
   - Enter home price, down payment, loan rate/term, holding period, and rent.
   - Add monthly costs (tax, insurance, repairs, HOA, PMI).
   - Enter transaction costs (buying and selling).
2. **Optional: Uncertainty**
   - Adjust market and cost distributions if you want stress-testing.
3. **Run Simulation**
   - The app will compute results and switch to the Summary view.

## Interpreting Results

- **Summary tab** shows the recommendation-style snapshot:
  - Probability buying is cheaper
  - Median savings vs rent
  - Risk indicators
- **Results tab** shows distributions and charts.
- **Comparison tab** focuses on buy vs rent.

## Assumptions and Limitations

This is a simplified model intended for clarity and decision support. It does **not** currently model:

- Tax deductions or credits
- Opportunity cost of invested cash
- Rent growth over time
- Variable-rate mortgages
- Extra principal payments or refinancing

The goal is to make tradeoffs and uncertainty visible, not to be a full underwriting model.

## Exporting Results

Use **File → Export Results** to save JSON or CSV for further analysis.

## Disclaimer

This software is provided for **informational and educational purposes only** and does **not** constitute financial, investment, legal, or tax advice. The simulations and results produced by this tool are based on simplified models and assumptions that may not reflect your actual financial situation. You should consult a qualified financial advisor before making any home-buying or other financial decisions.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
