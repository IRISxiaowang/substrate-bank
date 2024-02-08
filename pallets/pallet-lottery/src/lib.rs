//! # Lottery Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{BuildGenesisConfig, Randomness},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Saturating, Zero},
	DispatchResult, Percent,
};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*, vec::Vec};

use primitives::Role;
use traits::{BasicAccounting, GetTreasury, ManageRoles};

mod mock;
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;
	use sp_runtime::traits::BlockNumberProvider;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		/// The balance type
		type Balance: Member
			+ Parameter
			+ MaxEncodedLen
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ From<u128>
			+ sp_std::iter::Sum;

		type RoleManager: ManageRoles<Self::AccountId>;

		type Bank: BasicAccounting<Self::AccountId, Self::Balance> + GetTreasury<Self::AccountId>;

		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		#[pallet::constant]
		type LotteryPayoutPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type PrizePoolAccount: Get<Self::AccountId>;

		#[pallet::constant]
		type TaxRate: Get<Percent>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The prize split total is not equal to one.
		InvalidPrizeSplitTotal,
		/// The account role does not equal to the expected role.
		IncorrectRole,
		/// The ticket price is not set.
		TicketPriceNotSet,
		/// Tax rate must be between 0% - 100%.
		InvalidTaxRate,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Set the prize split.
		PrizeSplitUpdated { split: Vec<Percent> },
		/// Set the ticket price.
		TicketPriceUpdated { new: T::Balance },
		/// Customer bought numbers of tickets.
		TicketsBought { id: T::AccountId, number: u32, total_price: T::Balance },
		/// Customer won the lottery fund and paid tax.
		LotteryWon { user: T::AccountId, won_fund: T::Balance, tax: T::Balance },
	}

	#[pallet::storage]
	#[pallet::getter(fn prize_split)]
	pub type PrizeSplit<T: Config> = StorageValue<_, Vec<Percent>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn ticket_price)]
	pub type TicketPrice<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tickets)]
	pub type TicketsBought<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	pub(crate) type RandomSeed<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		phantom: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// Set PrizeSplit storage with Percent::one()
			PrizeSplit::<T>::put(vec![Percent::one()]);
		}
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			// check if we should payout this block
			if (block_number % T::LotteryPayoutPeriod::get()).is_zero() {
				let _ = Self::resolve_lottery_winner();
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the prize split.
		///
		/// Require root
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_prize_split())]
		pub fn set_prize_split(origin: OriginFor<T>, prize_split: Vec<Percent>) -> DispatchResult {
			ensure_root(origin)?;

			// Ensure the total of Percents adds up to Percent::one()
			ensure!(Self::check_split_valid(&prize_split), Error::<T>::InvalidPrizeSplitTotal);

			// Set the PrizeSplit storage
			PrizeSplit::<T>::put(prize_split.clone());

			Self::deposit_event(Event::<T>::PrizeSplitUpdated { split: prize_split });
			Ok(())
		}

		/// Set the ticket price.
		///
		/// Required Manager or Auditor.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_ticket_price())]
		pub fn update_ticket_price(origin: OriginFor<T>, new_price: T::Balance) -> DispatchResult {
			let id = ensure_signed(origin)?;
			let role = T::RoleManager::role(&id);
			ensure!(
				role == Some(Role::Manager) || role == Some(Role::Auditor),
				Error::<T>::IncorrectRole
			);

			// Update the ticket price
			TicketPrice::<T>::put(new_price);
			Self::deposit_event(Event::<T>::TicketPriceUpdated { new: new_price });

			Ok(())
		}

		/// Buy numbers of tickets and transfer the funds from the customer account to the lottery
		/// pool account.
		///
		/// Required Customer.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::buy_ticket())]
		pub fn buy_ticket(origin: OriginFor<T>, number_of_tickets: u32) -> DispatchResult {
			// Ensure the caller is a customer account
			let id = ensure_signed(origin)?;
			T::RoleManager::ensure_role(&id, Role::Customer)?;

			// Calculate the total price of the tickets
			ensure!(!TicketPrice::<T>::get().is_zero(), Error::<T>::TicketPriceNotSet);
			let total_price = TicketPrice::<T>::get().saturating_mul(number_of_tickets.into());

			// Transfer total_price from the customer to the PrizePoolAccount.
			T::Bank::transfer(&id, &T::PrizePoolAccount::get(), total_price)?;

			TicketsBought::<T>::mutate(id.clone(), |tickets| {
				*tickets = tickets.saturating_add(number_of_tickets)
			});

			Self::deposit_event(Event::<T>::TicketsBought {
				id: id.clone(),
				number: number_of_tickets,
				total_price,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Checks if the prize split adds up to 100% exactly.
		fn check_split_valid(split: &[Percent]) -> bool {
			split.iter().map(|x| *x * 100u32).sum::<u32>() == 100u32
		}

		/// Increase the random seed.
		fn next_seed() -> Vec<u8> {
			let seed = RandomSeed::<T>::get();
			RandomSeed::<T>::put(seed.wrapping_add(1));
			seed.encode()
		}

		/// Resolve the winner of a round of lottery. Transfer prize - tax to the winner and tax to
		/// the Treasury.
		fn resolve_lottery_winner() -> DispatchResult {
			// Make sure the treasury account is set.
			let treasury = T::Bank::treasury()?;

			// Ensure that the tax rate is less than or equal to 100% (1.0)
			let tax_rate = T::TaxRate::get();
			ensure!(tax_rate <= Percent::one(), Error::<T>::InvalidTaxRate);

			// Set up data for choosing winners
			// Get the total fund in the lottery pool.
			// Collect the players with the numbers of ticket bought for selecting winners, then
			// clear the TicketsBought storage for next round.
			let total = T::Bank::free_balance(&T::PrizePoolAccount::get());
			let players = TicketsBought::<T>::drain().collect::<Vec<_>>();

			// Choose the winners
			let winners = Self::select_n_winners(players, PrizeSplit::<T>::get().len() as u32);

			// Calculate the prize and tax amounts for each prize split percentage
			let (prize, tax): (Vec<T::Balance>, Vec<T::Balance>) = PrizeSplit::<T>::get()
				.into_iter()
				.map(|percent| {
					let full_prize = percent * total;
					((Percent::one() - tax_rate) * full_prize, tax_rate * full_prize)
				})
				.unzip();

			for (i, user) in winners.into_iter().enumerate() {
				// Payout the prize to each winner and put the tax into treasury account.
				let _ = T::Bank::transfer(&T::PrizePoolAccount::get(), &user, prize[i]);
				let _ = T::Bank::transfer(&T::PrizePoolAccount::get(), &treasury, tax[i]);

				// Added an event for each winner.
				Self::deposit_event(Event::<T>::LotteryWon {
					user,
					won_fund: prize[i],
					tax: tax[i],
				});
			}

			Ok(())
		}

		fn select_n_winners(
			mut players: Vec<(T::AccountId, u32)>,
			num_winners: u32,
		) -> Vec<T::AccountId> {
			let mut winners = vec![];

			// Choose n rounds of winners.
			for _ in 0..num_winners {
				// If numbers of chosen is larger than the number of players, then return.
				if players.is_empty() {
					break;
				}

				// Random select a winner in the players.
				let winner = Self::select_winner(players.clone());

				// Added the winner to the winner vec.
				winners.push(winner.clone());

				// Removed the chosen winner from the players vec.
				players.retain(|(player, _)| *player != winner);
			}

			winners
		}

		pub(crate) fn select_winner(players: Vec<(T::AccountId, u32)>) -> T::AccountId {
			let random_seed = Self::next_seed();
			let total: u32 = players.iter().map(|(_acc, n)| *n).sum();
			let (random, _) = T::Randomness::random(&random_seed);
			let target = <u32>::decode(&mut random.as_ref())
				.expect("hash should always be > 32 bits") %
				total;
			let mut sum = 0;

			// Find the winner who was holding the target number ticket.
			for (player, n) in players {
				sum += n;
				if sum > target {
					return player
				}
			}
			// It should never get here, if returned the pool account that means something is wrong.
			unreachable!();
			// T::PrizePoolAccount::get()
		}
	}
}
