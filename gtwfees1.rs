#![no_std]

elrond_wasm::imports!();

/// A contract that allows anyone to send a fixed sum, and dispatch to address.
/// Sending funds to the contract is called "ping".
/// Taking the same funds back is called "pong".
///
/// Restrictions:
/// - Only the set amount can be `ping`-ed, no more, no less.
/// - `pong` can only be called after a certain period after `ping`.
#[elrond_wasm::contract]
pub trait GtwFees1 {
    /// Necessary configuration when deploying:
    /// `min_amount` - The minimum value of token to be handle
    /// `fees_in_percent` - The value of fees to get from an amount in percent (e.g.: 12 for 12% of an amount in fees)
    /// `fees_addr` - ERD1 Address to receive fees
    /// `rest_addr` - ERD1 Addr to receive rest of payment
    /// `token_id` - Optional. The Token Identifier of the token that is going to be used. Default is "EGLD".
    #[init]
    fn init(
        &self,
        min_amount: BigUint,
        fees_in_percent: BigUint,
        fees_addr: ManagedAddress,
        rest_addr: ManagedAddress,
        #[var_args] opt_token_id: OptionalArg<TokenIdentifier>,
    ) -> SCResult<()> {
        require!(min_amount >= 0, "Min amount must be greater than or equal to zero");
        self.min_amount().set(&min_amount);
        require!(fees_in_percent > 0, "Fees in percent must be greater than zero");
        self.fees_in_percent().set(&fees_in_percent);
        let token_id = match opt_token_id {
            OptionalArg::Some(t) => t,
            OptionalArg::None => TokenIdentifier::egld(),
        };
        self.accepted_fees_addr_id().set(&fees_addr);
        self.accepted_rest_addr_id().set(&rest_addr);
        self.accepted_payment_token_id().set(&token_id);

        Ok(())
    }

    // endpoints

    /// User sends some tokens 
    /// Optional `_data` argument is ignored.
    #[payable("*")]
    #[endpoint]
    fn sendToken(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
    ) -> SCResult<()> {
        require!(
            payment_token == self.accepted_payment_token_id().get(),
            "Invalid payment token"
        );
        require!(
            payment_amount > self.min_amount().get(),
            "The payment must be greater than the min_amount"
        );

        let amount_fees = payment_amount.clone() * self.fees_in_percent().get() / BigUint::from(100u32);
        // let amount_fees = payment_amount.clone() / BigUint::from(10u32);
        let amount_rest = payment_amount.clone() - amount_fees.clone();

        self.send()
            .direct(&self.accepted_fees_addr_id().get(), &payment_token, 0, &amount_fees, b"fees from gtw sc");
        self.send()
            .direct(&self.accepted_rest_addr_id().get(), &payment_token, 0, &amount_rest, b"payment from gtw sc");

        Ok(())
    }

    // storage

    #[view(getAcceptedPaymentToken)]
    #[storage_mapper("acceptedPaymentTokenId")]
    fn accepted_payment_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getAcceptedFeesAddr)]
    #[storage_mapper("acceptedFeesAddrId")]
    fn accepted_fees_addr_id(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getAcceptedRestAddr)]
    #[storage_mapper("acceptedRestAddrId")]
    fn accepted_rest_addr_id(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getMinAmount)]
    #[storage_mapper("minAmount")]
    fn min_amount(&self) -> SingleValueMapper<BigUint>;

    
    #[view(feesInPercent)]
    #[storage_mapper("feesInPercent")]
    fn fees_in_percent(&self) -> SingleValueMapper<BigUint>;

}