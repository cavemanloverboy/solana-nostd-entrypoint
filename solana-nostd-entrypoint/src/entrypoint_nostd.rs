extern crate alloc;
use alloc::rc::Rc;

use core::{
    cell::RefCell,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ptr::NonNull,
    slice::from_raw_parts,
};

use {
    solana_program_entrypoint::{
        BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE, NON_DUP_MARKER,
    },
    solana_program_error::ProgramError,
    solana_program_memory::sol_memset,
    solana_pubkey::Pubkey,
};

#[macro_export]
macro_rules! entrypoint_nostd {
    ($process_instruction:ident, $accounts:literal) => {
        /// # Safety:
        /// solana entrypoint
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            // Create an array of uninitialized AccountInfos.
            const UNINIT_INFO: core::mem::MaybeUninit<
                NoStdAccountInfo,
            > = core::mem::MaybeUninit::uninit();
            let mut accounts = [UNINIT_INFO; $accounts];

            let (program_id, num_accounts, instruction_data) = unsafe {
                $crate::deserialize_nostd::<$accounts>(
                    input,
                    &mut accounts,
                )
            };

            let account_infos = core::slice::from_raw_parts(
                accounts.as_ptr() as *const NoStdAccountInfo,
                num_accounts,
            );

            match $process_instruction(
                &program_id,
                account_infos,
                &instruction_data,
            ) {
                Ok(()) => 0,
                Err(error) => error.into(),
            }
        }
    };
}

#[macro_export]
macro_rules! entrypoint_nostd_no_duplicates {
    ($process_instruction:ident, $accounts:literal) => {
        /// # Safety:
        /// solana entrypoint
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            // Create an array of uninitialized AccountInfos.
            const UNINIT_INFO: core::mem::MaybeUninit<
                NoStdAccountInfo,
            > = core::mem::MaybeUninit::uninit();
            let mut accounts = [UNINIT_INFO; $accounts];

            let Some((program_id, num_accounts, instruction_data)) =
                $crate::deserialize_nostd_no_dup::<$accounts>(
                    input,
                    &mut accounts,
                )
            else {
                // TODO: better error
                $crate::__private::sol_log(
                    "a duplicate account was found",
                );
                return u64::MAX;
            };

            let account_infos = core::slice::from_raw_parts(
                accounts.as_ptr() as *const NoStdAccountInfo,
                num_accounts,
            );

            match $process_instruction(
                &program_id,
                account_infos,
                &instruction_data,
            ) {
                Ok(()) => 0,
                Err(error) => error.into(),
            }
        }
    };
}

#[macro_export]
macro_rules! entrypoint_nostd_no_program {
    ($process_instruction:ident, $accounts:literal) => {
        /// # Safety:
        /// solana entrypoint
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            // Create an array of uninitialized AccountInfos.
            const UNINIT_INFO: core::mem::MaybeUninit<
                NoStdAccountInfo,
            > = core::mem::MaybeUninit::uninit();
            let mut accounts = [UNINIT_INFO; $accounts];

            let (num_accounts, instruction_data) = unsafe {
                $crate::deserialize_nostd_no_program::<$accounts>(
                    input,
                    &mut accounts,
                )
            };

            let account_infos = core::slice::from_raw_parts(
                accounts.as_ptr() as *const NoStdAccountInfo,
                num_accounts,
            );
            match $process_instruction(account_infos, &instruction_data)
            {
                Ok(()) => 0,
                Err(error) => error.into(),
            }
        }
    };
}

#[macro_export]
macro_rules! entrypoint_nostd_no_duplicates_no_program {
    ($process_instruction:ident, $accounts:literal) => {
        /// # Safety:
        /// solana entrypoint
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            // Create an array of uninitialized AccountInfos.
            const UNINIT_INFO: core::mem::MaybeUninit<NoStdAccountInfo> =
                core::mem::MaybeUninit::uninit();
            let mut accounts = [UNINIT_INFO; $accounts];

            let Some((num_accounts, instruction_data)) =
                $crate::deserialize_nostd_no_dup_no_program::<$accounts>(input, &mut accounts)
            else {
                // TODO: better error
                $crate::__private::sol_log("a duplicate account was found");
                return u64::MAX;
            };

            let account_infos = core::slice::from_raw_parts(
                accounts.as_ptr() as *const NoStdAccountInfo,
                num_accounts);
            match $process_instruction(
                account_infos,
                &instruction_data,
            ) {
                Ok(()) => 0,
                Err(error) => error.into(),
            }
        }
    };
}

/// # Safety
/// solana entrypoint
pub unsafe fn deserialize_nostd<'a, const MAX_ACCOUNTS: usize>(
    input: *mut u8,
    accounts: &mut [MaybeUninit<NoStdAccountInfo>],
) -> (&'a Pubkey, usize, &'a [u8]) {
    let mut offset: usize = 0;

    // Number of accounts present
    #[allow(clippy::cast_ptr_alignment)]
    let num_accounts = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let processed = if num_accounts > 0 {
        // we will only process up to MAX_ACCOUNTS
        let processed = num_accounts.min(MAX_ACCOUNTS);

        for i in 0..processed {
            let dup_info = *(input.add(offset) as *const u8);
            if dup_info == NON_DUP_MARKER {
                // MAGNETAR FIELDS: safety depends on alignment, size
                // 1) we will always be 8 byte aligned due to
                //    align_offset
                // 2) solana vm serialization format is consistent so
                //    size is ok
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;

                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch

                // MAGNETAR FIELDS: reset borrow state right before
                // pushing
                (*account_info).borrow_state = 0b_0000_0000;

                accounts[i].write(NoStdAccountInfo {
                    inner: account_info,
                });
            } else {
                offset += 8;
                // Duplicate account, clone the original
                accounts[i].write(
                    accounts[dup_info as usize]
                        .assume_init_ref()
                        .clone(),
                );
            }
        }

        // Skip any remaining accounts (if any) that we don't have space
        // to include.
        //
        // This duplicates the logic of parsing accounts but avoids the
        // extra CU consumption of having to check the array
        // bounds at each iteration.
        for _ in processed..num_accounts {
            if *(input.add(offset) as *const u8) == NON_DUP_MARKER {
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;
                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore
                                            // rent epoch
            } else {
                offset += 8;
            }
        }

        processed
    } else {
        // no accounts to process
        0
    };

    // Instruction data
    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len =
        *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let instruction_data =
        { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // Program Id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    (program_id, processed, instruction_data)
}

/// # Safety
/// solana entrypoint
pub unsafe fn deserialize_nostd_no_dup<
    'a,
    const MAX_ACCOUNTS: usize,
>(
    input: *mut u8,
    accounts: &mut [MaybeUninit<NoStdAccountInfo>],
) -> Option<(&'a Pubkey, usize, &'a [u8])> {
    let mut offset: usize = 0;

    // Number of accounts present
    #[allow(clippy::cast_ptr_alignment)]
    let num_accounts = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    // Account Infos
    #[allow(clippy::needless_range_loop)]
    let processed = if num_accounts > 0 {
        // we will only process up to MAX_ACCOUNTS
        let processed = num_accounts.min(MAX_ACCOUNTS);

        for i in 0..processed {
            let dup_info = *(input.add(offset) as *const u8);
            if dup_info == NON_DUP_MARKER {
                // MAGNETAR FIELDS: safety depends on alignment, size
                // 1) we will always be 8 byte aligned due to
                //    align_offset
                // 2) solana vm serialization format is consistent so
                //    size is ok
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;

                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch

                // MAGNETAR FIELDS: reset borrow state right before
                // pushing
                (*account_info).borrow_state = 0b_0000_0000;

                accounts[i].write(NoStdAccountInfo {
                    inner: account_info,
                });
            } else {
                return None;
            }
        }

        // Skip any remaining accounts (if any) that we don't have space
        // to include.
        //
        // This duplicates the logic of parsing accounts but avoids the
        // extra CU consumption of having to check the array
        // bounds at each iteration.
        for _ in processed..num_accounts {
            if *(input.add(offset) as *const u8) == NON_DUP_MARKER {
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;
                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore
                                            // rent epoch
            } else {
                offset += 8;
            }
        }

        processed
    } else {
        // there were not accounts on the input
        0
    };

    // Instruction data
    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len =
        *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let instruction_data =
        { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // Program Id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    Some((program_id, processed, instruction_data))
}

/// # Safety
/// solana entrypoint
pub unsafe fn deserialize_nostd_no_program<
    'a,
    const MAX_ACCOUNTS: usize,
>(
    input: *mut u8,
    accounts: &mut [MaybeUninit<NoStdAccountInfo>],
) -> (usize, &'a [u8]) {
    let mut offset: usize = 0;

    // Number of accounts present
    #[allow(clippy::cast_ptr_alignment)]
    let num_accounts = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    // Account Infos
    let processed = if num_accounts > 0 {
        // we will only process up to MAX_ACCOUNTS
        let processed = num_accounts.min(MAX_ACCOUNTS);

        for i in 0..processed {
            let dup_info = *(input.add(offset) as *const u8);
            if dup_info == NON_DUP_MARKER {
                // MAGNETAR FIELDS: safety depends on alignment, size
                // 1) we will always be 8 byte aligned due to
                //    align_offset
                // 2) solana vm serialization format is consistent so
                //    size is ok
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;

                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch

                // MAGNETAR FIELDS: reset borrow state right before
                // pushing
                (*account_info).borrow_state = 0b_0000_0000;

                accounts[i].write(NoStdAccountInfo {
                    inner: account_info,
                });
            } else {
                offset += 8;
                // Duplicate account, clone the original
                accounts[i].write(
                    accounts[dup_info as usize]
                        .assume_init_ref()
                        .clone(),
                );
            }
        }

        // Skip any remaining accounts (if any) that we don't have space
        // to include.
        //
        // This duplicates the logic of parsing accounts but avoids the
        // extra CU consumption of having to check the array
        // bounds at each iteration.
        for _ in processed..num_accounts {
            if *(input.add(offset) as *const u8) == NON_DUP_MARKER {
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;
                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore
                                            // rent epoch
            } else {
                offset += 8;
            }
        }

        processed
    } else {
        // there were not accounts on the input
        0
    };

    // Instruction data
    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len =
        *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let instruction_data =
        { from_raw_parts(input.add(offset), instruction_data_len) };

    (processed, instruction_data)
}

/// # Safety
/// solana entrypoint
pub unsafe fn deserialize_nostd_no_dup_no_program<
    'a,
    const MAX_ACCOUNTS: usize,
>(
    input: *mut u8,
    accounts: &mut [MaybeUninit<NoStdAccountInfo>],
) -> Option<(usize, &'a [u8])> {
    let mut offset: usize = 0;

    // Number of accounts present
    #[allow(clippy::cast_ptr_alignment)]
    let num_accounts = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    // Account Infos
    #[allow(clippy::needless_range_loop)]
    let processed = if num_accounts > 0 {
        // we will only process up to MAX_ACCOUNTS
        let processed = num_accounts.min(MAX_ACCOUNTS);

        for i in 0..processed {
            let dup_info = *(input.add(offset) as *const u8);
            if dup_info == NON_DUP_MARKER {
                // MAGNETAR FIELDS: safety depends on alignment, size
                // 1) we will always be 8 byte aligned due to
                //    align_offset
                // 2) solana vm serialization format is consistent so
                //    size is ok
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;

                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch

                // MAGNETAR FIELDS: reset borrow state right before
                // pushing
                (*account_info).borrow_state = 0b_0000_0000;

                accounts[i].write(NoStdAccountInfo {
                    inner: account_info,
                });
            } else {
                return None;
            }
        }

        // Skip any remaining accounts (if any) that we don't have space
        // to include.
        //
        // This duplicates the logic of parsing accounts but avoids the
        // extra CU consumption of having to check the array
        // bounds at each iteration.
        for _ in processed..num_accounts {
            if *(input.add(offset) as *const u8) == NON_DUP_MARKER {
                let account_info: *mut NoStdAccountInfoInner =
                    input.add(offset) as *mut _;
                offset += size_of::<NoStdAccountInfoInner>();
                offset += (*account_info).data_len;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8)
                    .align_offset(BPF_ALIGN_OF_U128);
                offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore
                                            // rent epoch
            } else {
                offset += 8;
            }
        }

        processed
    } else {
        // there were not accounts on the input
        0
    };

    // Instruction data
    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len =
        *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let instruction_data =
        { from_raw_parts(input.add(offset), instruction_data_len) };

    Some((processed, instruction_data))
}

#[derive(Clone, PartialEq, Eq)]
#[repr(C)]
pub struct NoStdAccountInfo {
    inner: *mut NoStdAccountInfoInner,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct NoStdAccountInfoInner {
    /// 0) We reuse the duplicate flag for this. We set it to
    ///    0b0000_0000.
    /// 1) We use the first four bits to track state of lamport borrow
    /// 2) We use the second four bits to track state of data borrow
    ///
    /// 4 bit state: [1 bit mutable borrow flag | u3 immmutable borrow
    /// flag] This gives us up to 7 immutable borrows. Note that
    /// does not mean 7 duplicate account infos, but rather 7 calls
    /// to borrow lamports or borrow data across all duplicate
    /// account infos.
    borrow_state: u8,

    /// Was the transaction signed by this account's public key?
    is_signer: u8,

    /// Is the account writable?
    is_writable: u8,

    /// This account's data contains a loaded program (and is now
    /// read-only)
    executable: u8,

    realloc_byte_counter: i32,

    /// Public key of the account
    key: Pubkey,
    /// Program that owns this account
    owner: Pubkey,

    /// The lamports in the account.  Modifiable by programs.
    lamports: u64,
    data_len: usize,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct AccountMetaC {
    // Public key of the account
    pub pubkey: *const Pubkey,

    // Is the account writable?
    pub is_writable: bool,

    // Transaction was signed by this account's key?
    pub is_signer: bool,
}

impl AccountMetaC {
    /// Uses signer = false. for signers use `readonly_signer`
    #[inline(always)]
    pub fn readonly(pubkey: *const Pubkey) -> AccountMetaC {
        AccountMetaC {
            pubkey,
            is_writable: false,
            is_signer: false,
        }
    }

    /// Uses signer = false. for signers use `writable_signer`
    #[inline(always)]
    pub fn writable(pubkey: *const Pubkey) -> AccountMetaC {
        AccountMetaC {
            pubkey,
            is_writable: true,
            is_signer: false,
        }
    }

    #[inline(always)]
    pub fn readonly_signer(pubkey: *const Pubkey) -> AccountMetaC {
        AccountMetaC {
            pubkey,
            is_writable: false,
            is_signer: true,
        }
    }

    #[inline(always)]
    pub fn writable_signer(pubkey: *const Pubkey) -> AccountMetaC {
        AccountMetaC {
            pubkey,
            is_writable: true,
            is_signer: true,
        }
    }
}

/// An AccountInfo as expected by sol_invoke_signed_c
#[repr(C)]
#[derive(Clone)]
pub struct AccountInfoC {
    // Public key of the account
    pub key: *const Pubkey,

    // Number of lamports owned by this account
    pub lamports: *const u64,

    // Length of data in bytes
    pub data_len: u64,

    // On-chain data within this account
    pub data: *const u8,

    // Program that owns this account
    pub owner: *const Pubkey,

    // The epoch at which this account will next owe rent
    pub rent_epoch: u64,

    // Transaction was signed by this account's key?
    pub is_signer: bool,

    // Is the account writable?
    pub is_writable: bool,

    // This account's data contains a loaded program (and is now
    // read-only)
    pub executable: bool,
}

impl AccountInfoC {
    /// A CPI utility function
    #[inline(always)]
    pub fn to_meta_c(&self) -> AccountMetaC {
        AccountMetaC {
            pubkey: self.key,
            is_writable: self.is_writable,
            is_signer: self.is_signer,
        }
    }

    /// A CPI utility function.
    /// Intended for PDAs that didn't sign transaction but must sign for
    /// cpi.
    #[inline(always)]
    pub fn to_meta_c_signer(&self) -> AccountMetaC {
        AccountMetaC {
            pubkey: self.key,
            is_writable: self.is_writable,
            is_signer: true,
        }
    }
}

/// An Instruction as expected by sol_invoke_signed_c
#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
pub struct InstructionC {
    /// Public key of the program
    pub program_id: *const Pubkey,

    /// Accounts expected by the program instruction
    pub accounts: *const AccountMetaC,

    /// Number of accounts expected by the program instruction
    pub accounts_len: u64,

    /// Data expected by the program instruction
    pub data: *const u8,

    /// Length of the data expected by the program instruction
    pub data_len: u64,
}

pub struct Ref<'a, T: ?Sized> {
    value: NonNull<T>,
    state: NonNull<u8>,
    is_lamport: bool,
    marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Ref<'a, T> {
    #[inline]
    pub fn map<U: ?Sized, F>(orig: Ref<'a, T>, f: F) -> Ref<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        // Avoid decrementing the borrow flag on Drop.
        let orig = ManuallyDrop::new(orig);

        Ref {
            value: NonNull::from(f(&*orig)),
            state: orig.state,
            is_lamport: orig.is_lamport,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn filter_map<U: ?Sized, F>(
        orig: Ref<'a, T>,
        f: F,
    ) -> Result<Ref<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        // Avoid decrementing the borrow flag on Drop.
        let orig = ManuallyDrop::new(orig);

        match f(&*orig) {
            Some(value) => Ok(Ref {
                value: NonNull::from(value),
                state: orig.state,
                is_lamport: orig.is_lamport,
                marker: PhantomData,
            }),
            None => Err(ManuallyDrop::into_inner(orig)),
        }
    }
}

impl<T: ?Sized> core::ops::Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T: ?Sized> Drop for Ref<'_, T> {
    // We just need to decrement the immutable borrow count
    // maybe super minor todo: we can save the is_lamport check by using
    // a separate ref type
    fn drop(&mut self) {
        if self.is_lamport {
            unsafe { *self.state.as_mut() -= 1 << 4 };
        } else {
            unsafe { *self.state.as_mut() -= 1 };
        }
    }
}

impl<T: ?Sized + core::fmt::Debug> core::fmt::Debug for Ref<'_, T> {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{:?}", &**self)
    }
}
pub struct RefMut<'a, T: ?Sized> {
    value: NonNull<T>,
    state: NonNull<u8>,
    is_lamport: bool,
    // `NonNull` is covariant over `T`, so we need to reintroduce
    // invariance.
    marker: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> RefMut<'a, T> {
    #[inline]
    pub fn map<U: ?Sized, F>(orig: RefMut<'a, T>, f: F) -> RefMut<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        // Avoid decrementing the borrow flag on Drop.
        let mut orig = ManuallyDrop::new(orig);

        RefMut {
            value: NonNull::from(f(&mut *orig)),
            state: orig.state,
            is_lamport: orig.is_lamport,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn filter_map<U: ?Sized, F>(
        orig: RefMut<'a, T>,
        f: F,
    ) -> Result<RefMut<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        // Avoid decrementing the mutable borrow flag on Drop.
        let mut orig = ManuallyDrop::new(orig);

        match f(&mut *orig) {
            Some(value) => {
                let value = NonNull::from(value);
                Ok(RefMut {
                    value,
                    state: orig.state,
                    is_lamport: orig.is_lamport,
                    marker: PhantomData,
                })
            }
            None => Err(ManuallyDrop::into_inner(orig)),
        }
    }
}

impl<T: ?Sized> core::ops::Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}
impl<T: ?Sized> core::ops::DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut <Self as core::ops::Deref>::Target {
        unsafe { self.value.as_mut() }
    }
}

impl<T: ?Sized> Drop for RefMut<'_, T> {
    // We need to unset the mut borrow flag
    // maybe super minor todo: we can save the is_lamport check by using
    // a separate type
    fn drop(&mut self) {
        if self.is_lamport {
            unsafe { *self.state.as_mut() &= 0b_0111_1111 };
        } else {
            unsafe { *self.state.as_mut() &= 0b_1111_0111 };
        }
    }
}

impl<T: ?Sized + core::fmt::Debug> core::fmt::Debug for RefMut<'_, T> {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{:?}", &**self)
    }
}

/// SAFETY:
/// Within the standard library, RcBox uses repr(C) which guarantees
/// we will always have the layout
///
/// strong: isize,
/// weak: isize,
/// value: T
///
/// For us, T -> RefCell<T>. Since RefCell<T> has T: ?Sized, this
/// guarantees that the inner fields of RefCell are not reordered.
/// So, in conclusion, this type has a stable memory layout.
#[repr(C, align(8))]
pub struct RcRefCellInner<'a, T> {
    strong: isize,
    weak: isize,
    refcell: RefCell<T>,
    phantom_data: PhantomData<&'a mut ()>,
}

impl<'a, T> RcRefCellInner<'a, T> {
    #[allow(unused)] // used for sol_invoke_signed_rust, which is commented out for
                     // now...
    fn new(value: T) -> RcRefCellInner<'a, T> {
        RcRefCellInner {
            strong: 2,
            weak: 2,
            refcell: RefCell::new(value),
            phantom_data: PhantomData,
        }
    }

    /// NOTE: when the last Rc is dropped, the strong count will reach
    /// one. So, it will not deallocate, which is fine because the
    /// Rc points to stack memory.
    ///
    /// SAFETY: [RcRefCellInner] must NOT be dropped before this Rc is
    /// used. There can be no safe abstraction that guarantees users
    /// do this because we cannot make Rc inherit the borrowed
    /// lifetime.
    #[allow(unused)] // used for sol_invoke_signed_rust, which is commented out for
                     // now...
    unsafe fn as_rcrc(&self) -> Rc<RefCell<T>> {
        // Rc::from_raw expects pointer to T
        unsafe { Rc::from_raw(&self.refcell as *const RefCell<T>) }
    }
}

#[inline(always)]
const fn offset<T, U>(ptr: *const T, offset: usize) -> *const U {
    unsafe { (ptr as *const u8).add(offset) as *const U }
}

impl NoStdAccountInfo {
    /// CPI utility function
    pub fn to_info_c(&self) -> AccountInfoC {
        AccountInfoC {
            key: offset(self.inner, 8),
            lamports: offset(self.inner, 72),
            data_len: self.data_len() as u64,
            data: offset(self.inner, 88),
            owner: offset(self.inner, 40),
            rent_epoch: 0,
            is_signer: self.is_signer(),
            is_writable: self.is_writable(),
            executable: self.executable(),
        }
    }

    /// CPI utility function
    pub fn to_meta_c(&self) -> AccountMetaC {
        AccountMetaC {
            pubkey: offset(self.inner, 8),
            is_writable: self.is_writable(),
            is_signer: self.is_signer(),
        }
    }

    /// CPI utility function.
    ///
    /// Intended for pdas that did not sign transaction but need to sign
    /// for cpi.
    pub fn to_meta_c_signer(&self) -> AccountMetaC {
        AccountMetaC {
            pubkey: offset(self.inner, 8),
            is_writable: self.is_writable(),
            is_signer: true,
        }
    }

    // These two functions can be used to cpi via
    // sol_invoke_signed_rust, but it is very easy to mess this up.
    // Please just use sol_invoke_signed_c.
    //
    // pub unsafe fn unchecked_info_prep<'a>(
    //     &'a self,
    // ) -> (RcRefCellInner<&'a mut u64>, RcRefCellInner<&'a mut [u8]>)
    // {     let lamports_inner =
    // RcRefCellInner::new(self.unchecked_borrow_mut_lamports());
    //     let data_inner =
    // RcRefCellInner::new(self.unchecked_borrow_mut_data());
    //     (lamports_inner, data_inner)
    // }
    // pub unsafe fn info_with<'a>(
    //     &'a self,
    //     lamports_data: &'a (RcRefCellInner<&'a mut u64>,
    // RcRefCellInner<&'a mut [u8]>),
    // ) -> solana_program::account_info::AccountInfo<'a> {
    //     let (lamports, data) = lamports_data;
    //     solana_program::account_info::AccountInfo {
    //         key: self.key(),
    //         lamports: unsafe { lamports.as_rcrc() },
    //         data: unsafe { data.as_rcrc() },
    //         owner: self.owner(),
    //         rent_epoch: u64::MAX,
    //         is_signer: self.is_signer(),
    //         is_writable: self.is_writable(),
    //         executable: self.executable(),
    //     }
    // }

    #[inline(always)]
    pub fn key(&self) -> &Pubkey {
        unsafe { &(*self.inner).key }
    }
    #[inline(always)]
    pub fn owner(&self) -> &Pubkey {
        unsafe { &(*self.inner).owner }
    }
    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        unsafe { (*self.inner).is_signer != 0 }
    }
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        unsafe { (*self.inner).is_writable != 0 }
    }
    #[inline(always)]
    pub fn executable(&self) -> bool {
        unsafe { (*self.inner).executable != 0 }
    }
    #[inline(always)]
    pub fn data_len(&self) -> usize {
        unsafe { (*self.inner).data_len }
    }

    /// # Safety
    /// This does not check or modify the 4-bit refcell. Useful when
    /// instruction has verified non-duplicate accounts.
    pub unsafe fn unchecked_borrow_lamports(&self) -> &u64 {
        &(*self.inner).lamports
    }
    /// # Safety
    /// This does not check or modify the 4-bit refcell. Useful when
    /// instruction has verified non-duplicate accounts.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn unchecked_borrow_mut_lamports(&self) -> &mut u64 {
        &mut (*self.inner).lamports
    }
    /// # Safety
    /// This does not check or modify the 4-bit refcell. Useful when
    /// instruction has verified non-duplicate accounts.
    pub unsafe fn unchecked_borrow_data(&self) -> &[u8] {
        core::slice::from_raw_parts(
            self.data_ptr(),
            (*self.inner).data_len,
        )
    }
    /// # Safety
    /// This does not check or modify the 4-bit refcell. Useful when
    /// instruction has verified non-duplicate accounts.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn unchecked_borrow_mut_data(&self) -> &mut [u8] {
        core::slice::from_raw_parts_mut(
            self.data_ptr(),
            (*self.inner).data_len,
        )
    }

    /// Tries to get a read only reference to the lamport field, failing
    /// if the field is already mutable borrowed or if 7 borrows
    /// already exist.
    pub fn try_borrow_lamports(
        &self,
    ) -> Result<Ref<u64>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if mutable borrow is already taken
        if *borrow_state & 0b_1000_0000 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // Check if we have reached the max immutable borrow count
        if *borrow_state & 0b_0111_0000 == 0b_0111_0000 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // Increment the immutable borrow count
        *borrow_state += 1 << 4;

        // Return the reference to lamports
        Ok(Ref {
            value: unsafe { NonNull::from(&(*self.inner).lamports) },
            state: unsafe {
                NonNull::new_unchecked(&mut (*self.inner).borrow_state)
            },
            is_lamport: true,
            marker: PhantomData,
        })
    }

    /// Tries to get a read only reference to the lamport field, failing
    /// if the field is already borrowed in any form.
    pub fn try_borrow_mut_lamports(
        &self,
    ) -> Result<RefMut<u64>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if any borrow (mutable or immutable) is already taken
        // for lamports
        if *borrow_state & 0b_1111_0000 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // Set the mutable lamport borrow flag
        *borrow_state |= 0b_1000_0000;

        // Return the mutable reference to lamports
        Ok(RefMut {
            value: unsafe {
                NonNull::new_unchecked(&mut (*self.inner).lamports)
            },
            state: unsafe {
                NonNull::new_unchecked(&mut (*self.inner).borrow_state)
            },
            is_lamport: true,
            marker: PhantomData,
        })
    }

    /// Tries to get a read only reference to the data field, failing if
    /// the field is already mutable borrowed or if 7 borrows
    /// already exist.
    pub fn try_borrow_data(&self) -> Result<Ref<[u8]>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if mutable data borrow is already taken (most
        // significant bit of the data_borrow_state)
        if *borrow_state & 0b_0000_1000 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // Check if we have reached the max immutable data borrow count
        // (7)
        if *borrow_state & 0b0111 == 0b0111 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // Increment the immutable data borrow count
        *borrow_state += 1;

        // Return the reference to data
        Ok(Ref {
            value: unsafe {
                NonNull::from(core::slice::from_raw_parts(
                    self.data_ptr(),
                    (*self.inner).data_len,
                ))
            },
            state: unsafe {
                NonNull::new_unchecked(&mut (*self.inner).borrow_state)
            },
            is_lamport: false,
            marker: PhantomData,
        })
    }

    /// Tries to get a read only reference to the data field, failing if
    /// the field is already borrowed in any form.
    pub fn try_borrow_mut_data(
        &self,
    ) -> Result<RefMut<[u8]>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if any borrow (mutable or immutable) is already taken
        // for data
        if *borrow_state & 0b_0000_1111 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // Set the mutable data borrow flag
        *borrow_state |= 0b0000_1000;

        assert_eq!(self.data_ptr() as usize % 8, 0); // TODO REMOVE

        // Return the mutable reference to data
        Ok(RefMut {
            value: unsafe {
                NonNull::new_unchecked(core::slice::from_raw_parts_mut(
                    self.data_ptr(),
                    (*self.inner).data_len,
                ))
            },
            state: unsafe {
                NonNull::new_unchecked(&mut (*self.inner).borrow_state)
            },
            is_lamport: false,
            marker: PhantomData,
        })
    }

    /// Realloc the account's data and optionally zero-initialize the
    /// new memory.
    ///
    /// Note:  Account data can be increased within a single call by up
    /// to `solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE`
    /// bytes.
    ///
    /// Note: Memory used to grow is already zero-initialized upon
    /// program entrypoint and re-zeroing it wastes compute units.
    /// If within the same call a program reallocs from larger to
    /// smaller and back to larger again the new space could contain
    /// stale data.  Pass `true` for `zero_init` in this case,
    /// otherwise compute units will be wasted re-zero-initializing.
    ///
    /// # Safety
    ///
    /// This method makes assumptions about the layout and location of
    /// memory referenced by `AccountInfo` fields. It should only be
    /// called for instances of `AccountInfo` that were created by
    /// the runtime and received in the `process_instruction`
    /// entrypoint of a program.
    pub fn realloc(
        &self,
        new_len: usize,
        zero_init: bool,
    ) -> Result<(), ProgramError> {
        let mut data = self.try_borrow_mut_data()?;
        let old_len = data.len();

        // Return early if length hasn't changed
        match new_len.cmp(&old_len) {
            // Nothing to do
            core::cmp::Ordering::Equal => return Ok(()),

            // No checks
            // Old len fits in an i32, and new_len is smaller so new len
            // fits in i32.
            core::cmp::Ordering::Less => {
                unsafe {
                    (*self.inner).realloc_byte_counter -=
                        (old_len - new_len) as i32;

                    // Set new length in the serialized data
                    (*self.inner).data_len = new_len;
                };
            }

            // Need to check that the diff fits in an i32 and that max
            // realloc has not been reached
            core::cmp::Ordering::Greater => {
                unsafe {
                    // Check diff
                    let new_counter = (*self.inner)
                        .realloc_byte_counter
                        + TryInto::<i32>::try_into(new_len - old_len)
                            .map_err(|_| ProgramError::InvalidRealloc)?;

                    // Check to see if we've exceeded max realloc across
                    // all invocations
                    if new_counter > MAX_PERMITTED_DATA_INCREASE as i32
                    {
                        return Err(ProgramError::InvalidRealloc);
                    }

                    // Set new length and new counter in the serialized
                    // data after all validation
                    (*self.inner).data_len = new_len;
                    (*self.inner).realloc_byte_counter = new_counter;

                    // Zero init if specified
                    if zero_init {
                        let len_increase = new_len - old_len;
                        let new_data_slice =
                            core::slice::from_raw_parts_mut(
                                data.as_mut_ptr().add(old_len),
                                len_increase,
                            );
                        sol_memset(new_data_slice, 0, len_increase);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn reassign(&self, new_owner: &Pubkey) {
        // Set the non-mut owner field
        unsafe {
            (*self.inner).owner = *new_owner;
        }
    }

    /// Private: gets the memory addr of the account data
    fn data_ptr(&self) -> *mut u8 {
        unsafe {
            (self.inner as *const _ as *mut u8)
                .add(size_of::<NoStdAccountInfoInner>())
        }
    }
}

#[test]
fn test_ref() {
    let lamports_data: [u8; 8] =
        unsafe { core::mem::transmute([0u64; 1]) };
    let borrow_state = 1 << 4;
    let byte_ref: Ref<[u8; 8]> = Ref {
        value: NonNull::from(&lamports_data),
        state: NonNull::from(&borrow_state),
        is_lamport: true,
        marker: PhantomData,
    };

    let lamports_ref: Ref<u64> = Ref::map(byte_ref, |b| unsafe {
        core::mem::transmute::<&[u8; 8], &u64>(b)
    });
    assert_eq!(borrow_state, 1 << 4);
    assert_eq!(*lamports_ref, 0_u64);

    let odd_lamports_ref = Ref::filter_map(lamports_ref, |b| {
        if *b % 2 == 1 {
            Some(b)
        } else {
            None
        }
    });
    assert_eq!(borrow_state, 1 << 4);
    assert!(odd_lamports_ref.is_err());
    let lamports_ref = odd_lamports_ref.unwrap_err();
    assert_eq!(*lamports_ref, 0_u64);

    let even_lamports_ref = Ref::filter_map(lamports_ref, |b| {
        if *b % 2 == 0 {
            Some(b)
        } else {
            None
        }
    });
    assert_eq!(borrow_state, 1 << 4);
    assert_eq!(*even_lamports_ref.unwrap(), 0_u64);
}

#[test]
fn test_ref_mut() {
    let lamports_data: [u8; 8] =
        unsafe { core::mem::transmute([0u64; 1]) };
    let borrow_state = 1 << 4;
    let byte_ref: RefMut<[u8; 8]> = RefMut {
        value: NonNull::from(&lamports_data),
        state: NonNull::from(&borrow_state),
        is_lamport: true,
        marker: PhantomData,
    };

    let lamports_ref: RefMut<u64> = RefMut::map(byte_ref, |b| unsafe {
        core::mem::transmute::<&mut [u8; 8], &mut u64>(b)
    });
    assert_eq!(borrow_state, 1 << 4);
    assert_eq!(*lamports_ref, 0_u64);

    let odd_lamports_ref = RefMut::filter_map(lamports_ref, |b| {
        if *b % 2 == 1 {
            Some(b)
        } else {
            None
        }
    });
    assert_eq!(borrow_state, 1 << 4);
    assert!(odd_lamports_ref.is_err());

    let mut lamports_ref = odd_lamports_ref.unwrap_err();
    assert_eq!(*lamports_ref, 0_u64);
    *lamports_ref += 2;
    assert_eq!(*lamports_ref, 2_u64);

    let even_lamports_ref = RefMut::filter_map(lamports_ref, |b| {
        if *b % 2 == 0 {
            Some(b)
        } else {
            None
        }
    });
    assert_eq!(borrow_state, 1 << 4);
    assert_eq!(*even_lamports_ref.unwrap(), 2_u64);
}
