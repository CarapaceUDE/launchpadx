# Windows Code Signing

LaunchPadX embeds its icon and version metadata regardless of signing. Removing Windows SmartScreen's unknown-publisher warning requires a publicly trusted Authenticode identity; a self-signed certificate is useful only for local testing.

## Recommended: Microsoft Azure Artifact Signing

Microsoft Artifact Signing keeps signing keys in managed hardware and its Public Trust profile is intended for publicly distributed Win32 applications. The Basic tier currently costs $9.99 USD per month and includes 5,000 signatures. Public Trust organization validation is available to US organizations such as Carapace LLC.

Setup:

1. Create a paid Azure subscription associated with Carapace LLC.
2. Follow Microsoft's [Artifact Signing quickstart](https://learn.microsoft.com/azure/artifact-signing/quickstart) to create a Basic account, complete organization identity validation, and create a **Public Trust** certificate profile.
3. Configure GitHub-to-Azure OpenID Connect using the official [`azure/login`](https://github.com/Azure/login) instructions. Grant the identity the **Artifact Signing Certificate Profile Signer** role.
4. Add these GitHub Actions secrets:

   - `AZURE_CLIENT_ID`
   - `AZURE_TENANT_ID`
   - `AZURE_SUBSCRIPTION_ID`
   - `AZURE_ARTIFACT_SIGNING_ENDPOINT`
   - `AZURE_ARTIFACT_SIGNING_ACCOUNT`
   - `AZURE_ARTIFACT_SIGNING_PROFILE`

The release workflow detects those secrets, signs both Windows executables with Microsoft's official action, timestamps them, and rebuilds the ZIP.

## Alternative: trusted PFX certificate

If a trusted certificate provider gives you an exportable Authenticode PFX, add:

- `WINDOWS_CERTIFICATE_BASE64`: the PFX bytes encoded as Base64
- `WINDOWS_CERTIFICATE_PASSWORD`: the PFX password

The workflow signs with `signtool`, applies an RFC 3161 timestamp, verifies both executables, and deletes the temporary certificate file. Never commit a PFX or password to the repository.

## Verification

After downloading a release on Windows:

```powershell
Get-AuthenticodeSignature .\launchpadx.exe | Format-List Status,SignerCertificate,TimeStamperCertificate
```

`Status` must be `Valid`. Signing establishes publisher identity and integrity; it does not replace release smoke tests or malware scanning.
