def sign_in(user, password):
    # §FS-login.requirements.audit
    audit(user)
    if user.failures >= 5:
        # §FS-login.requirements.lockout
        raise Locked(user)
    return check(user, password)


def reset_link(user):
    # §FS-login.requirements.reset
    return Link(user, hours=1)
