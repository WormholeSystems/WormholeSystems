---
title: Moving from the legacy app
---

# Moving from the legacy app

The old app keeps running at [legacy.wormhole.systems](https://legacy.wormhole.systems).
Your data does not move over on its own: there is no account or database migration. You
bring each map across yourself. Export it from the legacy app as a file, then import that
file here.

The file carries the map itself: systems, connections, signatures, notes, settings, the
watch list, the ignore list and the access list. Everything tied to you as a person stays
behind. You sign in again, connect your characters again and set up Discord again.

## Step by step

1. **Sign in here.** Click **Log in with EVE Online** and sign in with the character you
   use on the legacy map. Accounts are not shared between the two apps, so this is a fresh
   account. Add your other characters and grant scopes as described in
   [Connecting your character](/documentation/getting-started/connecting-your-character).
2. **Export the map on legacy.** Open the map on legacy.wormhole.systems, click
   **Settings** in the status bar, then **Import / Export** in the sidebar. On the
   **Export** tab every section is selected: **Map settings**, **Access**, **Solar
   systems**, **Connections**, **Signatures** and **Routes**. Leave them all on and click
   **Download export**. You need to be a manager or the owner of the map to see this page.
3. **Bring it in here.** You have two choices.
   - **As a new map.** On the **Maps** page click **Import**, next to **New map**. Choose
     the file. The name is filled in from the file and you can change it. Keep the sections
     ticked and click **Import**. The new map opens, and you are its owner.
   - **Into a map you already have here.** Open the map, click the gear (**Map settings**)
     and go to **Import and export**. Under **Import into this map**, choose the file, pick
     the sections and click **Import**. You need Manager access. When it is done, a summary
     shows how many entries each section created, updated or skipped.
4. **Check who has access.** Open **Map settings** and then **Access**. The grants from
   legacy are there, but the owner is whoever did the import. If the map should belong to
   someone else, hand it over under **General**. See
   [How access works](/documentation/access-and-permissions/how-access-works).
5. **Set up the personal side again.** Turn on location sharing for the map
   ([Character presence](/documentation/tracking/character-presence)), link your Discord
   account under **Settings**, **Discord**, and recreate your
   [Discord alerts](/documentation/intel/alerts). Make the map public or create a share link
   again if you used one
   ([Public maps & share links](/documentation/access-and-permissions/public-maps-and-share-links)).

Do this once per map. Each of your pilots only needs to sign in and do step 5 for
themselves.

## What carries over

| On legacy                                                          | Here                                                                                                                                               |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Systems, with position, alias, pin, status, occupier and notes     | Come over. A system that only has notes on legacy (no position) keeps its notes and status but is not placed on the map.                           |
| Connections, with mass, lifetime, ship size, preserve mass and age | Come over between systems that are on the map. A connection's wormhole type sets its size; the type name shows once a linked signature carries it. |
| Signatures                                                         | Come over, still linked to their connections. A signature without a full scanner id (like `ABC-123`) is skipped, because this app needs the id.    |
| Access for characters, corporations and alliances                  | Comes over with each role and expiry date. The owner does not: whoever imports the map owns it here.                                               |
| Name, layout, bookmark formats, naming scheme                      | Come over. The constant width display setting does not exist here and is ignored.                                                                  |
| Home and rally points                                              | Come over when the system is on the map. Only the newest two rally points are kept.                                                                |
| Watch list and ignore list                                         | Come over. On a new map they replace the trade hubs that are normally pre-loaded.                                                                  |
| Public map and share link                                          | Do not come over. Set them up again.                                                                                                               |
| Discord alerts, webhooks and roles                                 | Do not come over. Set them up again.                                                                                                               |
| Your characters, scopes and location sharing                       | Do not come over. Sign in and connect them again.                                                                                                  |
| Linked Discord account, API tokens                                 | Do not come over. Link and create them again.                                                                                                      |
| Personal display settings, background image                        | Do not come over.                                                                                                                                  |
| Map history, ship history, jump log                                | Do not come over.                                                                                                                                  |

The names for mass, lifetime and ship size are the same in both apps (fresh, reduced,
critical; healthy, EOL, critical, expired; frigate up to extra large). An import of the same
file twice is safe: systems, signatures and grants update in place, and a connection that
already exists is left alone. Importing into an existing map is recorded as one entry in the
map history and cannot be undone.

Killmails are not part of the file. This app collects its own, so the kill feed works
without them.

## When the import refuses

If something is wrong with the file, the import stops before changing anything and says
why in a short message.

- **this file is not valid JSON** or **this file is not a wormholesystems map export**: the
  file is not a map export. Pick the `.json` file that **Download export** saved, usually
  named after the map with `-export-` and the date.
- **this file was exported by an incompatible version of the application**: the file says
  it uses a format version this app does not read. Export it again from legacy.
- **the file does not contain any sections**: the export was made with nothing selected.
  Export again with the sections on.
- **the file does not contain a valid map name**: the map name in the file is empty or
  longer than 255 characters.
- **the "signatures" section contains invalid data** (or another section), or **an export
  never contains an owner grant**: the file was changed after it was exported. Export it
  again.
- **(needs Systems)** next to Connections or Signatures: a new map needs the systems for its
  connections and signatures to hang off. Tick **Systems** as well.
- **No Import button on the Maps page**: this instance only lets certain characters,
  corporations or alliances create maps. Ask someone who can to create the map, have them
  give you Manager access, and import into it from **Import and export**.

Files can be up to 32 MB, which covers even large alliance chains.

## Nothing is lost by trying

The legacy app stays up at [legacy.wormhole.systems](https://legacy.wormhole.systems) and an
export does not change the map there. If something looks wrong, delete the imported map
here and try again, or keep flying on legacy while you sort it out.

## Self-hosting

The same export and import works between any two instances, so you can move a map from
your own legacy install, or from one instance of this app to another, one map at a time.
There is no way to import a legacy database (MySQL) into this app's database (Postgres),
and no plan to add one. See [Self-hosting](/documentation/contributing-and-self-hosting/self-hosting).
