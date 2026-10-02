# REL-schema-later: pay-as-you-go structure, schema added where it pays

Pay-as-you-go data management asks for no schema investment up front and adds structure
only where it pays off. It is the precedent for a grund repository that starts with no
configuration and adds kinds and rules as it grows ([§GOAL-small-and-large](../goals.md#goal-small-and-large-start-small-configure-for-big)).

## work: What the work is

Michael Franklin, Alon Halevy and David Maier, *From Databases to Dataspaces: A New
Abstraction for Information Management*, SIGMOD Record 34(4):27–33, December 2005, DOI
10.1145/1107499.1107502
([SIGMOD Record](https://sigmodrecord.org/2005/12/04/from-databases-to-dataspaces-a-new-abstraction-for-information-management/)).
A dataspace support platform provides "base functionality over all data sources,
regardless of how integrated they are", and where more is needed, effort "can be applied
to more closely integrate those sources in an incremental, 'pay-as-you-go' fashion". Its
queries "may return best-effort or approximate answers".

## agrees: Where grund agrees

- A repository starts with no investment: a conformant tree works with no config and no
  flags ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)), and a small one without ceremony ([§GOAL-small-and-large.1](../goals.md#1-small-repo-promise)).
- Structure is added where it pays: kinds, citation directions and chapter rules are
  opt-in settings in `grund.toml`, not modes the tool switches on by itself
  ([§GOAL-small-and-large.3](../goals.md#3-layout-knobs-live-in-config)).

## departs: Where grund departs

- A dataspace leaves its sources in their own formats and integrates across them. grund
  integrates no foreign source: an outside fact enters only as a committed declaration in
  its own scheme ([§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).
- A dataspace may answer best-effort. grund's answer never is: what a run reads it checks
  in full, and two installs agree on every byte of it ([§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree),
  [§FS-non-goals.14.1](../functional-spec/FS-non-goals.md#141-token-saving-inside-check)).
