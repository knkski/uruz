from runelib import StateReaction, Action


# Noop handler, returning `None` means proceed with default behavior
# Should probably emit a warning and prefer an explicit approval, though
def react_noop(action: Action) -> StateReaction:
    ...


# Example handler showing an exception being thrown. Handled as
# try:
#     react_error(new_state, old_state, diff)
# except Exception as err:
#     return ChangeFailed('State change handler failed.', err)
def react_fail(action: Action) -> StateReaction:
    raise Exception("Something happened!")


# Example handler that always denies a state change
def react_deny(action: Action) -> StateReaction:
    return StateReaction.denied("I don't like that change very much.")


# Example handler that always approves a state change
def react_approve(action: Action) -> StateReaction:
    return StateReaction.approved


# Example handler that checks something about the new state, and denies
# the change request if it finds something it doesn't like. The plain
# strings below would be passed to a SQL library IRL.
def react_check(action: Action) -> StateReaction:
    if new_state.config["user"] == "root":
        return ChangeDenied("Non-root user can't be set to root!")

    if old_state.config["user"]:
        existing_user = f"SELECT * FROM users WHERE name = {old_state.config['user']}"
        if not existing_user:
            print("WARN: Old user doesn't exist!")

    try:
        f"UPDATE users SET name = {new_state.config['user']} WHERE name = {old_state.config['user']}"
    except Exception as err:
        return StateReaction.failed("Couldn't update username", err)

    return StateReaction.handled
