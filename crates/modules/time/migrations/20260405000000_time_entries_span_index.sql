-- The longest stored entry bounds how far back a Period's query must look (repo::entries::overlapping);
-- this index makes max(stop - start) a constant-time lookup instead of a scan.
CREATE INDEX time_entries_span_idx ON time.time_entries ((stop - start));
