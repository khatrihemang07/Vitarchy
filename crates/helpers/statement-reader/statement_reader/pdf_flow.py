"""PDF -> pages -> header (or inferred columns) -> regions -> rows -> Statements."""

from __future__ import annotations

from .contract import Diagnostics, Result, Statement
from .locate.headerless import infer_layouts
from .locate.regions import Region, headerless_regions, locate_regions
from .log import stage
from .metadata.surface import Surface, pdf_surface
from .normalize.rows import NormaliseError
from .read.lines import Line, group_lines
from .read.pdf_words import Page, PageReader
from .statements import group_regions, make_statement, segment_group, statement_facts
from .vocabulary import Vocabulary, load_vocabulary


def _pages(reader: PageReader) -> list[Page]:
    return [reader.read_page(i) for i in range(reader.page_count)]


def _statements(regions: list[Region], page_lines: list[list[Line]], vocab: Vocabulary) -> list[Statement]:
    statements = []
    doc_surface = pdf_surface([line for lines in page_lines for line in lines])
    groups = group_regions(regions)
    firsts = [min(r.page for r in group) for group in groups]
    start = 0
    for index, group in enumerate(groups):
        # A statement owns the pages from the end of the previous one up to the start of the next:
        # its figures may sit on a cover before the table or in a summary box after it.
        end = firsts[index + 1] if index + 1 < len(groups) else len(page_lines)
        surface: Surface = pdf_surface([line for n in range(start, end) for line in page_lines[n]])
        start = max(r.page for r in group) + 1
        facts = statement_facts(surface, group[0].heading, vocab)
        if not (facts.issuer and facts.kind):  # a cover page outside this statement's pages may name them
            doc_facts = statement_facts(doc_surface, None, vocab)
            facts.issuer = facts.issuer or doc_facts.issuer
            facts.kind = facts.kind or doc_facts.kind
        statements.append(make_statement(segment_group(group, vocab), facts, doc_surface, vocab))
    return statements


def _chain_ok(statements: list[Statement]) -> bool:
    chains = [c for s in statements for c in s.checks if c.name == "balance_chain"]
    return bool(statements) and all(c.outcome == "passed" for c in chains) and bool(chains)


def _headerless(pages: list[Page], page_lines, vocab: Vocabulary) -> list[Statement]:
    """The first inferred layout (debit/credit order guessed) whose balance chain holds."""
    for layout in infer_layouts([line for lines in page_lines for line in lines], vocab):
        regions = headerless_regions(pages, layout, vocab)
        statements = _statements(regions, page_lines, vocab) if regions else []
        if _chain_ok(statements):
            return statements
    return []


def read_pdf(reader: PageReader) -> Result:
    vocab = load_vocabulary()
    pages = _pages(reader)
    page_lines = [group_lines(p.words, n) for n, p in enumerate(pages)]
    diagnostics = Diagnostics(pages=len(pages))
    try:
        with stage("locate") as counts:
            regions = locate_regions(pages, vocab)
            statements = _statements(regions, page_lines, vocab) if regions else []
            if not any(s.transactions for s in statements):
                statements = _headerless(pages, page_lines, vocab)
            counts.update(regions=len(regions), statements=len(statements))
    except NormaliseError as error:
        return Result(outcome="no_table", diagnostics=Diagnostics("normalize", error.code, len(pages)))
    statements = [s for s in statements if s.transactions]
    if not statements:
        failure = Diagnostics("locate", "no_header_or_columns", len(pages))
        return Result(outcome="no_table", diagnostics=failure)
    diagnostics.rows = sum(len(s.transactions) for s in statements)
    return Result(outcome="read", kind="statement", statements=statements, diagnostics=diagnostics)
