/// useful if you want to figure out if the the environment or our code is at fault for having
/// problems with saving passwords
#[test]
fn test_keyring_save_read_delete() -> keyring::Result<()> {
	let entry = keyring::Entry::new("test-service", "test@example.com")?;

	entry.set_password("42")?;

	let password = entry.get_password()?;
	assert_eq!(password, "42");

	entry.delete_credential()?;

	assert!(entry.get_password().is_err());

	Ok(())
}
