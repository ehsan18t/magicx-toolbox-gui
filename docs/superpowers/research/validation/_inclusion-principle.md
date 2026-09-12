# What belongs in the corpus

The inclusion rule for tweaks, and the correction to how this review was applying it.

## The principle

**The product exists to give the user control over their own machine.** Whether Microsoft ships a
setting on or off is not the product's concern and is not an inclusion criterion.

If a real, toggleable control exists on the supported Windows range, it belongs in the corpus. The
user decides what to do with it. The app's job is to surface the control, describe it honestly, and
make it reversible.

## The single test

> Does a real control exist that changes something on Windows 11 24H2 or newer?

If yes, include it.

If no, exclude it, because there is nothing to control.

That is the whole test. Not "is it on by default", not "does applying it change anything today", not
"would most users want it".

## What this replaces

Earlier in this review, entries were demoted or questioned with reasoning like "already the default,
so this is a no-op", "just confirms the current state", or "applying it changes nothing". That
reasoning was a category error. It judged the *default* when the product's subject is the *control*.

An earlier draft of this document then over-corrected, arguing at length for why pinning a default is
still valuable. That was also wrong, in a subtler way: it accepted the premise that a tweak whose
setting already matches the default needs a justification to exist. It does not. It needs a control
to exist, and that is all.

## Where the default state still matters

Exactly two places, both about correctness rather than inclusion:

1. **The revert value.** The System Default option must write the state the machine would be in
   without the tweak. Getting this wrong is the corpus's most damaging defect class, tracked in
   `_harmful-revert.md`. This is why defaults still have to be researched carefully. It is a
   correctness question, not an inclusion question.

2. **Honest description.** If applying a tweak produces no visible change today, the `info` block
   should say so, so the user is not left hunting for a difference that is not there. Stating that
   plainly is honest. Using it as a reason to omit the control is not.

## What this does not change

The 12 deletions in `_rescope-24h2.md` still stand. Those fail the test for the right reason: the
feature was removed from Windows, so there is no control left to expose. The Cortana button, Meet
Now, and the Chat button do not exist on 24H2 in any state, on or off.

The distinction is simple. "Windows ships this off" means a control exists and is currently off, so
include it. "Windows removed this" means no control exists, so exclude it.

Two of the twelve are worth re-checking against this test before deletion, because their surface may
still exist even though the feature is deprecated:

- `disable_people_bar`, since `PeopleBand.dll` still ships on 26100
- `disable_news_interests`, whose Windows 10 LTSC 2021 applicability was never confirmed

## Consequence for the entries

No tweak is added or removed by this principle beyond the re-check noted above. Twelve entries carry
framing that needs rewriting, listed below. The mechanism findings for all of them stand; only the
prose changes, and it gets shorter.

`security:disable_wdigest`, `security:block_vulnerable_drivers`, `security:disable_lmhash_storage`,
`security:printnightmare_point_and_print`, `security:dotnet_strong_crypto`, `security:disable_smb_guest`,
`security:require_smb_signing`, `security:remove_smbv1`, `security:enable_lsa_protection`,
`privacy:disable_device_name_in_telemetry`, `services:disable_smartcard`,
`performance:ntfs_disable_lastaccess`.

For each, drop the "this is a no-op" or "already the default" editorializing from the recommendation.
State what the control does, note in Drawbacks if there is no visible change today, and let the user
choose.

## One product implication

The UI should distinguish **"currently at the Windows default"** from **"not applied"**. They read
the same today but mean different things to someone deciding what to do, and the engine already
detects the state it needs to tell them apart.
