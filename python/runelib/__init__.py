"""Library that defines classes for Uruz-Rune interactions.

A handler may die suddenly without warning due to any number of reasons,
in which case Uruz will attempt to run the handler again with the same
arguments, until it gets a `ChangeApproved` or `ChangeHandled` response.
This may result in the function having to handle a partial state change,
in which case the handler may log a warning and update state anyways, or
just deny the change and tell an administrator that they have to straighten
things out.
"""

from enum import Enum


class Action:
    ...


class StateReaction(Enum):
    """Reaction to a proposed state change."""

    """Signifies that the rune code approved the state change.

    The code didn't handle the change itself though, and Uruz should
    proceed with default state change update handling.
    """

    approved = int()

    """Signifies that the rune code approved and updated the state."""
    handled = 2

    """Signifies that the rune code denied updating the state.

    Uruz will not attempt this change again
    """
    denied = 3

    """Signifies that the rune code failed updating the state.

    Uruz will attempt this change again until it either succeeds or
    fails with `ChangeDenied`.
    """
    failed = 4


# Classes in rune.state or somesuch
class State:
    def __init__(self):
        self.config = {}
        self.relations = {}
        self.uruz = {}


class Diff:
    def __init__(self):
        self.config = {}
        self.relations = {}
        self.uruz = {}
