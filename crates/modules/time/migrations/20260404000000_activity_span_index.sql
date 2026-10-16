-- The longest stored row bounds how far back a Period's query must look (repo::activities::overlapping);
-- this index makes max(end_at - start_at) a constant-time lookup instead of a scan.
CREATE INDEX activities_span_idx ON time.activities ((end_at - start_at));
