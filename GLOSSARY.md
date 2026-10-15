# Vitarchy

A personal life-management app, built up from self-contained areas of life (time, money, and so on) that are added one at a time.

## Language

**Module**:
A self-contained area of life management, such as Time or Finance, that owns its records and its screens.
_Avoid_: Plugin, feature, app, domain

**Core**:
The shared ground that Modules may rely on. It holds the Settings that apply across all of Vitarchy, and no other domain concepts.
_Avoid_: Common, shared, base, kernel

**Setting**:
A value the person chooses that changes how Vitarchy looks or behaves, such as the theme or the first day of the week. Every Setting has a default and belongs to exactly one scope: Core or a single Module.
_Avoid_: Preference, option, config

**Core settings**:
The Settings that apply across all of Vitarchy: display name, theme, motion, week start, clock and date format.
_Avoid_: Common settings, global settings, general settings

**Module settings**:
The Settings that belong to one Module and only change that Module.
_Avoid_: Plugin settings, feature flags
