# Security

Do not put credentials in a catalog, issue, fixture or pull request. Report a
sensitive issue using the repository's private vulnerability reporting feature
when available; otherwise contact the maintainer privately before publication.

External snapshots are data, not trusted instructions. `Catalog::from_json`
validates schema, bounds, route uniqueness, and allowed patch keys, but cannot
certify the truth of externally supplied provider claims. Obtain snapshots from
trusted, reviewed sources. Apply fragments through a validated provider adapter.

The library never fetches a URL, launches a process, loads credentials, writes
settings, or changes a requested model ID. Unknown combinations return `None`.
