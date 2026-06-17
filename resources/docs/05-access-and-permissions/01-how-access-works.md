---
title: How access works
---

# How access works

Access is managed **per map** and is **hierarchical** — each level includes everything the level below it can do.

Each access entry grants a level to **one** subject:

- a **character**
- a **corporation**
- an **alliance**

When someone opens a map, the app gathers every non-expired entry that matches their character, its corporation, and its alliance, and grants them the **highest** level among them. So a pilot whose character is only a Viewer but whose corp has Manager gets Manager.

Each entry can also carry an optional **expiration date**, after which it simply stops counting.

## Owner

The pilot who creates a map is its **owner**. The owner is the only one who can **delete** the map, and the owner's own access entry can't be edited or removed by anyone.

## Who can manage access

Only **Managers** (and the owner) can see and edit the access list. Viewers and Members can use the map but can't change who else has access.

The next pages cover the [three roles](/documentation/access-and-permissions/roles) and [public maps and share links](/documentation/access-and-permissions/public-maps-and-share-links).
