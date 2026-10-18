-- Each card on a bill is its own Account, so a Transaction no longer carries an add-on card.
ALTER TABLE finance.transactions DROP COLUMN addon_card;
