# rig-dynclient

This crate provides a dynamic client on top of rig that selects model providers
at run-time.

Provider clients are initialized using environment variables. See rig's
documentation for specifics.

> [!note]
>This was extracted from [rig-core](https://github.com/0xPlaygrounds/rig/blob/rig-core-v0.31.0/rig/rig-core) at the suggestion of rig maintainers.
>
> It fixes issues with the original implementation by using enum dispatch
> instead of downcasting.
