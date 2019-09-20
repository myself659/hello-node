/// A runtime module template with necessary imports
// Encoding library
use parity_codec::Encode;

// Enables access to store a value in runtime storage
// Imports the `Result` type that is returned from runtime functions
// Imports the `decl_module!` and `decl_storage!` macros
use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageValue};

// Traits used for interacting with Substrate's Balances module
// `Currency` gives you access to interact with the on-chain currency
// `WithdrawReason` and `ExistenceRequirement` are enums for balance functions
use support::traits::{Currency, ExistenceRequirement, WithdrawReason};

// Enables us to verify an call to our module is signed by a user account
use system::ensure_signed;

// These are traits which define behavior around math and hashing
use runtime_primitives::traits::{Hash, Saturating, Zero};
pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as hello {
        Payment get(payment): Option<T::Balance>;
        Pot get(pot): T::Balance;
        Nonce get(nonce): u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;
	// This function initializes the `payment` storage item
        // It also populates the pot with an initial value
        fn set_payment(origin, value: T::Balance) -> Result {
            // Ensure that the function call is a signed message (i.e. a transaction)
            let _ = ensure_signed(origin)?;
            // If `payment` is not initialized with some value
            if Self::payment().is_none() {
                // Set the value of `payment`
                <Payment<T>>::put(value);
                // Initialize the `pot` with the same value
                <Pot<T>>::put(value);
            // Raise event for the set payment
            Self::deposit_event(RawEvent::PaymentSet(value));
            }
            // Return Ok(()) when everything happens successfully
            Ok(())
        }

        // This function is allows a user to play our coin-flip game
        fn play(origin) -> Result {
            // Ensure that the function call is a signed message (i.e. a transaction)
            // Additionally, derive the sender address from the signed message
            let sender = ensure_signed(origin)?;
            // Ensure that `payment` storage item has been set
            let payment = Self::payment().ok_or("Must have payment amount set")?;
            // Read our storage values, and place them in memory variables
            let mut nonce = Self::nonce();
            let mut pot = Self::pot();

            // Try to withdraw the payment from the account, making sure that it will not kill the account
            let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender, payment, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;
            let mut winnings = Zero::zero();

            // Generate a random hash between 0-255 using a csRNG algorithm
            if (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash)
                .using_encoded(|e| e[0] < 128)
            {
                // If the user won the coin flip, deposit the pot winnings; cannot fail
                let _ = <balances::Module<T> as Currency<_>>::deposit_into_existing(&sender, pot)
                .expect("`sender` must exist since a transaction is being made and withdraw will keep alive; qed.");
        // Set the winnings
        winnings = pot;
                // Reduce the pot to zero
                pot = Zero::zero();
            }

            // No matter the outcome, increase the pot by the payment amount
            pot = pot.saturating_add(payment);
            // Increment the nonce
            nonce = nonce.wrapping_add(1);

            // Store the updated values for our module
            <Pot<T>>::put(pot);
            <Nonce<T>>::put(nonce);

            // Raise event for the play result
            Self::deposit_event(RawEvent::PlayResult(sender, winnings));
            // Return Ok(()) when everything happens successfully
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
    {
        PaymentSet(Balance),
        PlayResult(AccountId, Balance),
    }
);
