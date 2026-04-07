/* [074A-22] Unique constraint on domain for fixture system and business logic.
 * PG allows multiple NULLs in unique columns so hosting without domain is fine. */
ALTER TABLE hosting_subscriptions
    ADD CONSTRAINT hosting_subscriptions_domain_key UNIQUE (domain);
