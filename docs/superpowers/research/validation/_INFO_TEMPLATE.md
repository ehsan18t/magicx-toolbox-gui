# Tweak `info` template

The required shape of every tweak's `info` block. One skeleton, every tweak, no exceptions. A reader
should know what a tweak does, what it costs, and whether to apply it, in about five seconds of
scanning, without reading a paragraph.

Stored as markdown inside the existing `info` field, so this needs no schema or frontend change.

## The skeleton

````yaml
    info: |
      **One sentence: what you get, in plain language. Bold, no heading above it.**

      ## What it does
      Two or three sentences maximum. Name the actual mechanism (the service, the value, the
      policy) so a technical reader can verify it. No marketing.

      ## Benefits
      - **Short label**: one clause of detail
      - **Short label**: one clause of detail

      ## Drawbacks
      - **Short label**: one clause of detail
      - **Short label**: one clause of detail

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately | after sign-out | after restarting Explorer | after reboot
      - **Reverting**: restores the previous value from the snapshot
      - Any caveat that does not fit the two lists above

      ## Recommendation
      One or two sentences. Say who should apply it and who should not. Take a position.

      ## Evidence
      - **Risk**: low | medium | high | critical
      - **Confidence**: Microsoft-documented | Community-corroborated
      - [Source title](https://url)
      - [Source title](https://url)
````

## Rules

1. **The hook is one sentence and states the payoff**, not the mechanism. It is the only line most
   people read. Bold, and never preceded by a heading.
2. **Benefits and drawbacks are 2 to 4 bullets each.** Never more than four. If there are more,
   the extras were not important.
3. **Every bullet leads with a bolded 1 to 3 word label**, then a colon, then one clause. The label
   carries the meaning on its own, so the list is scannable without reading the details.
4. **Drawbacks is never omitted.** If a tweak genuinely has none, write a single bullet:
   `- **None of consequence**: cosmetic and fully reversible.` A tweak with no stated cost reads as
   marketing and gets trusted less, not more.
5. **"Takes effect" is mandatory and must be accurate.** This is the single most common support
   question and the corpus currently gets it wrong in places. It must match the tweak's
   `requires_reboot` flag.
6. **Recommendation takes a position.** "Depends on your needs" is not a recommendation. Name the
   user who should apply it and the user who should not.
7. **Risk in the Evidence block must match the tweak's `risk_level` field.** Two sources of truth
   that disagree is worse than one.
8. **Confidence is one of exactly two values.** `Microsoft-documented` means a Microsoft Learn page,
   shipped ADMX, or KB confirms the mechanism. `Community-corroborated` means three or more
   genuinely independent sources agree but Microsoft does not document it. The second is not a
   warning label; much of Windows is legitimately undocumented.
9. **Two to four sources.** Cite the best ones, not all of them. Prefer the primary source first.
10. **No em dashes anywhere.** Project rule.
11. **Do not lose information.** Where the current prose carries a real caveat, it moves into
    Drawbacks or Good to know. Shortening is the goal; deleting facts is not.

## Worked example

The before and after for `performance:disable_search_indexing`. Same facts, roughly 40 percent
fewer words, scannable in one pass.

### Before

> **Stops the Windows Search background indexer, cutting idle disk and CPU churn on slow or low-RAM
> machines.**
>
> ## What it does
> Disables the Windows Search service (`WSearch`), the background process that continuously catalogs
> your files so Start Menu and File Explorer searches return instantly. With it stopped, Windows
> **stops building and maintaining the search index**, ending the associated idle disk and CPU activity.
>
> ## What you gain
> You remove a background task that quietly reads and writes to disk and uses CPU while the system is
> idle. **On low-RAM machines or spinning hard drives, cutting that churn can make the system feel
> less busy**, and it is redundant if you already use a faster tool like Everything.
>
> ## Good to know
> **Start Menu and File Explorer search become slow, scanning on demand instead of hitting a prebuilt
> index.** On a modern SSD with plenty of RAM the gain is *marginal*: the indexer typically writes
> under 20 MB a day and uses under 1% idle CPU.
>
> ## Recommendation
> **Use it only on an HDD or low-RAM system, or if you have replaced Windows Search with something
> like Everything.** On a healthy SSD machine, leave it on.

### After

````yaml
    info: |
      **Stops the background file indexer, cutting idle disk and CPU activity.**

      ## What it does
      Disables the `WSearch` service, which continuously catalogs your files so Start and File
      Explorer searches return instantly. With it stopped, Windows no longer builds or maintains
      the search index.

      ## Benefits
      - **Less idle disk I/O**: no background indexing while you are not using the PC
      - **Frees CPU**: indexing spikes noticeably on large libraries
      - **Redundant with Everything**: no reason to index twice if you use a third-party search

      ## Drawbacks
      - **Slow file search**: Explorer scans on demand instead of reading a prebuilt index
      - **Start menu search**: finding apps and settings becomes slower and less complete
      - **Outlook search breaks**: classic Outlook depends on the Windows index
      - **Marginal on modern hardware**: under 20 MB written per day and under 1 percent idle CPU

      ## Good to know
      - **Applies to**: Windows 11 24H2 and newer
      - **Takes effect**: immediately, the service stops on apply
      - **Reverting**: restores the previous start type from the snapshot

      ## Recommendation
      Worth it on a hard drive or a low-RAM machine, or if you already search with Everything. On a
      healthy SSD with adequate RAM, leave it enabled; the gain does not justify losing search.

      ## Evidence
      - **Risk**: medium
      - **Confidence**: Microsoft-documented
      - [Windows Search overview](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
      - [WSearch service defaults](https://learn.microsoft.com/en-us/windows-server/security/windows-services/security-guidelines-for-disabling-system-services-in-windows-server)
````

## Applying this

Do the rewrite **after** the second verification round and the gap hunt land, not before. Twelve
tweaks are slated for deletion, roughly a hundred have pending corrections, and new tweaks may be
added. Rewriting first means rewriting twice.
