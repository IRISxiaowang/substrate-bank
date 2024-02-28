#![cfg(test)]

use frame_support::{assert_ok, traits::Hooks};
use sp_runtime::{BuildStorage, Percent};

use primitives::{constants::*, AccountId, Balance, Role, DAY};
use traits::{BasicAccounting, Stakable};

#[allow(unused_imports)]
use xy_chain_runtime::{
	Bank, Governance, Lottery, Roles, Runtime, RuntimeCall, RuntimeError, RuntimeEvent,
	RuntimeOrigin, System, Timestamp,
};

mod bank;
mod lottery;

pub const INITIAL_BALANCE: Balance = 1_000_000 * DOLLAR;
pub const TICKETPRICE: Balance = DOLLAR;

macro_rules! test_account {
	($name:ident, $id:expr) => {
		pub struct $name;

		impl $name {
			pub fn account(&self) -> AccountId {
				$id.into()
			}

			pub fn sign(&self) -> RuntimeOrigin {
				RuntimeOrigin::signed(self.account())
			}

			pub fn to_string(&self) -> String {
				format!(
					"{}: 0x{}",
					stringify!($name),
					$id.iter().map(|byte| { format!("{:02x}", byte) }).collect::<String>()
				)
			}
		}

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				// Convert the array of bytes to a hexadecimal string
				write!(f, "{}", self.to_string())
			}
		}
	};
}

// Customers
test_account!(Alice, [0x00; 32]);
test_account!(Bob, [0x01; 32]);
test_account!(Charlie, [0x02; 32]);
test_account!(Dave, [0x03; 32]);
test_account!(Eve, [0x04; 32]);

// Bank manager + auditor
test_account!(Manager, [0xFA; 32]);
test_account!(Auditor, [0xFB; 32]);

// Default governance members
test_account!(Gov0, [0xF0; 32]);
test_account!(Gov1, [0xF1; 32]);
test_account!(Gov2, [0xF2; 32]);

// Default treasury account
test_account!(Treasury, [0xE0; 32]);

pub struct ExtBuilder {
	governance_members: Vec<AccountId>,
	balances: Vec<(AccountId, Balance)>,
	roles: Vec<(AccountId, Role)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			governance_members: vec![Gov0.account(), Gov1.account(), Gov2.account()],
			balances: vec![
				(Alice.account(), INITIAL_BALANCE),
				(Bob.account(), INITIAL_BALANCE),
				(Charlie.account(), INITIAL_BALANCE),
				(Dave.account(), INITIAL_BALANCE),
				(Eve.account(), INITIAL_BALANCE),
			],
			roles: vec![
				(Manager.account(), Role::Manager),
				(Auditor.account(), Role::Auditor),
				(Alice.account(), Role::Customer),
				(Bob.account(), Role::Customer),
				(Charlie.account(), Role::Customer),
				(Dave.account(), Role::Customer),
				(Eve.account(), Role::Customer),
			],
		}
	}
}

impl ExtBuilder {
	pub fn governance_members(mut self, members: Vec<AccountId>) -> Self {
		self.governance_members = members;
		self
	}

	pub fn balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn roles(mut self, roles: Vec<(AccountId, Role)>) -> Self {
		self.roles = roles;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		// Build storage
		let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

		pallet_bank::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_roles::GenesisConfig::<Runtime> { roles: self.roles }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_lottery::GenesisConfig::<Runtime>::default()
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_governance::GenesisConfig::<Runtime> {
			initial_authorities: self.governance_members,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t.build_storage().unwrap());

		ext.execute_with(|| {
			System::set_block_number(1);
			pallet_bank::TreasuryAccount::<Runtime>::set(Some(Treasury.account()));
			assert_ok!(Lottery::update_ticket_price(Manager.sign(), DOLLAR));
		});

		ext
	}
}

// Contains test utility functions
#[track_caller]
pub fn assert_balance(who: AccountId, amount: Balance) {
	assert_eq!(Bank::free_balance(&who), amount);
}

#[track_caller]
pub fn assert_staked(who: AccountId, amount: Balance) {
	assert_eq!(Bank::staked(&who), amount);
}
