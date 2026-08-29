# CI/CD

ReVault is designed to make it easy to share secrets with a CI/CD platform.

We strongly recommend that you use different secrets for Dev, CI/CD, Staging and Production platforms and keep each set of secrets in a different lockbox using a different profile for each.&#x20;

This guide makes the assumption that you keep the suggested separation. You can however (not recommended) use a single lockbox using variable paths such as /dev/API\_TOKEN and /staging/API\_TOKEN. &#x20;

### Profiles

An profile is a named set of keys.  If we are to create a lockbox for each environment (CI/staging/production...) then we need a separate profile for each environment.

There is  a secondary issue that needs to be address.  We don't recommend sharing vaults between systems and since need to create a profile for each system that means that we need to create a vault for each system.   These vaults however need to know something about each other.



Really these vaults should not share keys with the dev team as that breaks isolation.&#x20;

So let's start by creating the necessary profiles.

We assume that you already have a vault initialised on your local dev PC. If not read the [quick start guide](cli-tooling/quick-start-guide.md).

### Create a profile



