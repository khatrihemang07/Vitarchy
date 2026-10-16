# Vitarchy

A personal life-management app, built up from self-contained areas of life (time, money, and so on) that are added one at a time.

## Language

**Module**:
A self-contained area of life management, such as Time or Finance, that owns its records and its screens.
_Avoid_: Plugin, feature, app, domain

**Core**:
The shared ground that Modules may rely on. It holds the Settings that apply across all of Vitarchy, and no other domain concepts.
_Avoid_: Common, shared, base, kernel

**Live**:
The environment holding the person's real data, used day to day (`make start`). Never used for tests or automated checks.
_Avoid_: Prod, production, dev database

**Stage**:
The throwaway environment, with its own RAM-only database, where all tests and agent checks run (`make test`, `make smoke`, `make stage`).
_Avoid_: Test database, staging

**Setting**:
A value the person chooses that changes how Vitarchy looks or behaves, such as the theme or the first day of the week. Every Setting has a default and belongs to exactly one scope: Core or a single Module.
_Avoid_: Preference, option, config

**Core settings**:
The Settings that apply across all of Vitarchy: display name, theme, motion, week start, clock and date format.
_Avoid_: Common settings, global settings, general settings

**Module settings**:
The Settings that belong to one Module and only change that Module.
_Avoid_: Plugin settings, feature flags
