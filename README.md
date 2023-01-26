# TrueNAS Netcup DNS ACME

Small binary that can be used as an ACME Auth inside TrueNAS

Uses the Netcup DNS API to set and unset the DNS records

## How it works

TrueNAS Scale allows you to define an ACME DNS-Auth using a shell script, but this functionality seems to be poorly documented (i.e. the only thing I could find was the comment at the top of the sourcefile implementing the feature: https://github.com/truenas/middleware/blob/master/src/middlewared/middlewared/plugins/acme_protocol_/authenticators/shell.py)

The script gets called at the start of the authentication process and at the end and only has one job:

Set and delete the TXT DNS-Record so the certificate authority can verify ownership.

## Usage

Place binary in a convenient location.

Create a `config.json` next to it with the following content:

```json
{
  "CID": "Customer ID",
  "API_PW": "API Password",
  "API_KEY": "Api Key"
}
```

Create a new ACME DNS-Authenticator in TrueNAS, set it's type to `shell`, point it to the executable. Set other options as necessary.

After that just use the normal certificate request process to create your certificates.

## Possible todo's

Maybe enhance this to support other DNS providers? I don't know if this feature is even used that heavily, but let me know if you happen to be in the same situation as me and need something like this.

Maybe create a rust port of https://github.com/AnalogJ/lexicon ?

## Why rust?

My first working version of this was written as a shell-script. It was not pretty... (Neither is this, but at least I understand it)
