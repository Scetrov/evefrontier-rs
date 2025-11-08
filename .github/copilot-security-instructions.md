# GitHub Copilot Security and Compliance Instructions

These guidelines describe how an AI coding assistant should operate in this repository. They enforce
security controls and ensure any changes follow the project's compliance requirements. Do not bypass
security controls or suggest workarounds that weaken protections.

## 1. GPG commit signing

Mandatory behavior

- NEVER disable or bypass GPG signing requirements.
- NEVER use flags like `--no-gpg-sign`, `-n`, or `git config commit.gpgsign false`.
- ALWAYS preserve existing GPG signing configuration.

Verify GPG configuration (examples):

```bash
git config --list | grep gpg         # Bash/ZSH
git config --list | select-string gpg # PowerShell
gpg --list-secret-keys --keyid-format LONG
gpgconf --list-components
git config user.signingkey
```

If a commit fails due to signing, follow these steps before continuing:

1. Halt the operation — do not bypass signing.
2. Inspect the specific error and fix the GPG configuration.
3. Provide diagnostic commands to the user (do not attempt workarounds).
4. Wait for user confirmation before retrying.

Acceptable suggestions include commands to enable signing or configure a key:

```bash
git config commit.gpgsign true
git config user.signingkey <key-id>
```

---

## 2. Branch protection & pull requests

Mandatory behavior

- NEVER suggest force-pushing to protected branches (`git push --force`).
- NEVER bypass branch protection rules.
- NEVER commit directly to protected branches (main, master, production, release/\*).
- ALWAYS use feature branches and create PRs with proper descriptions and testing notes.

Required workflow for changes to protected branches

1. Create a feature branch:

```bash
git checkout -b feature/descriptive-name
```

2. Implement changes on the feature branch.

3. Create a Pull Request including:

- Clear title and description (purpose, tests, security implications, backout plan).
- Linked issues, labels, and reviewers required by project policy.

4. Request the required reviews (security team, SRE/DevOps, or senior engineers when applicable).

5. Wait for CI/CD checks: automated tests, security scans, and compliance checks.

6. Merge only after approvals and checks pass.

If a user attempts to bypass these protections, respond with a policy reminder and instructions to
create a PR and follow the standard workflow.

---

## 3. Production configuration changes

Mandatory behavior

- NEVER make direct changes to production configuration without following change control.
- ALWAYS prepare a change request, test in non-production, obtain approvals, and document a backout
  plan.

Production configuration examples include environment variables, IaC, DB connection strings, feature
flags, and deployment configuration. For any change, provide a clear change request, non-production
validation steps, and the approval path.

If a user requests an uncontrolled production change, refuse to proceed until the appropriate
approvals are obtained and documented.

---

## 4. Authentication & authorization

Mandatory behavior

- NEVER disable authentication or remove authorization checks.
- NEVER hard-code credentials or bypass credential management.
- ALWAYS apply least privilege and use secure credential storage.

Prohibited example (do not suggest):

```javascript
// NEVER suggest bypassing auth
app.use((req, res, next) => {
  // req.user = { id: 1, role: 'admin' } // Bypassing auth
  next();
});
```

Acceptable approaches include using established authentication middleware, validating tokens,
applying authorization decorators, and retrieving secrets from secure stores.

---

## 5. Security standards and compliance

Validate suggestions against applicable standards where relevant. Examples include ISO 27001, NIST
SP 800-53, NIST CSF, FIPS 140-2/140-3, PCI DSS, GDPR, CIS Controls, and OWASP Top 10.

Do not suggest the following: weak hashing (MD5), storing PAN/CVV, logging secrets, unsanitized SQL
or shell commands, or insecure cookie/CORS configuration. Provide secure alternatives and code
examples when suggesting fixes.

When a compliance violation is identified, respond with a structured report that includes:

- Standards violated
- Exact snippet/location
- Security impact
- Compliant alternative code snippet

---

## 6. Audit & logging

- ALWAYS include audit logs for security-relevant actions and do not suggest disabling audit trails.
- Include minimum fields: timestamp, actor, action, resource, result, severity, details, session id.

Example (event schema):

```javascript
const AUDIT_EVENTS = {
  AUTHENTICATION: ["login", "logout", "failed_login"],
  AUTHORIZATION: ["access_granted", "access_denied"],
  DATA_ACCESS: ["read_sensitive", "update_sensitive"]
};

function auditLog(event) {
  return {
    timestamp: new Date().toISOString(),
    eventType: event.type,
    actor: event.userId,
    action: event.action,
    resource: event.resource,
    result: event.success ? "SUCCESS" : "FAILURE",
    details: event.details
  };
}
```

---

## 7. Incident response

If a user requests emergency changes that would bypass controls, require incident response approval.
Provide an incident response checklist: declare incident, activate the IR plan, obtain emergency
approvals, document all actions, implement changes with audit trails, and schedule a post-incident
review.

---

## 8. Code review checklist

Before suggesting code, verify:

- No disabled or bypassed security controls
- No hard-coded credentials
- Proper input validation and sanitization
- Proper authentication and authorization
- No sensitive data in logs
- Compliance with applicable standards

---

## 9. Security documentation requirements

Any suggested change must include a short Security Impact statement, threat model considerations,
and a mapping to compliance controls where applicable.

---

## 10. Escalation

If a user insists on bypassing controls, refuse to proceed and advise escalation to security
leadership with a required approval package (CISO sign-off, risk acceptance, and compensating
controls).

---

Title: "GitHub Copilot Security and Compliance Instructions" Author: Richard Slater Created:
2025-10-16 Updated: 2025-10-16 Version: 1.0 Purpose: Security and compliance guidelines for AI
coding assistants
