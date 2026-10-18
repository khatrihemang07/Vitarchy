import pytest

from statement_reader.segment.wrap import join_description


@pytest.mark.parametrize("lines,joined", [
    (["FAKE PERSON 1", "UPI/600000000000/NA/fake0@"], "FAKE PERSON 1 UPI/600000000000/NA/fake0@"),
    (["UPI/600000000000/NA/fake0@", "exb/Example Bank/EXB0c3c5d", "0a1b2e3f4a5"],
     "UPI/600000000000/NA/fake0@exb/Example Bank/EXB0c3c5d0a1b2e3f4a5"),
    (["UPI/DR/512345678901/DEMO GRO/HDFC/xyautopa", "y./Indi"],
     "UPI/DR/512345678901/DEMO GRO/HDFC/xyautopay./Indi"),
    (["UPI-512345679054-Example Marketing Pv", "IN"], "UPI-512345679054-Example Marketing Pv IN"),
    (["UPI-512345679071-ANITA", "NAIR IN"], "UPI-512345679071-ANITA NAIR IN"),
    (["AMAZON PAY IN E COMMERC BANGALORE", "IN"], "AMAZON PAY IN E COMMERC BANGALORE IN"),
    (["HDFC", "BANK"], "HDFC BANK"),
    (["PAYMENT TO ACME", "FOR ORDER 12345"], "PAYMENT TO ACME FOR ORDER 12345"),
])
def test_wrapped_lines_join_with_a_space_unless_the_wrap_split_an_id_token(lines, joined):
    assert join_description(lines) == joined


def test_blank_lines_are_dropped_and_whitespace_runs_collapse():
    assert join_description(["  WDL   TFR ", "", "UPI/1  "]) == "WDL TFR UPI/1"
    assert join_description([]) == ""


def test_a_lone_hex_fragment_after_a_long_id_is_glued_but_a_word_is_not():
    long_id = "UPI/Asha Rao/5123456993@ibl/loan/StateBank/512345679088/ICIe898dbce8d364025a3bf3493259dc4e"
    assert join_description([long_id, "a"]) == long_id + "a"
    assert join_description(["UPI/DEMOFOOD/512345679122", "NAIR"]) == "UPI/DEMOFOOD/512345679122 NAIR"


def test_a_short_line_is_never_glued_to_the_next_one():
    lines = ["UPI/DEMO CLINIC/democlinicx/Refund/HDFC", "BANK/512345679139/HDF68EBCDF9B3CC43208C313F27421"]
    assert join_description(lines, [False, True]).startswith("UPI/DEMO CLINIC/democlinicx/Refund/HDFC BANK/")
    assert join_description(lines, [True, True]).startswith("UPI/DEMO CLINIC/democlinicx/Refund/HDFCBANK/")


def test_a_full_line_ending_in_a_joiner_continues_into_the_next_word():
    assert join_description(["XLa9ec7652b512345933b806fea3bbb2cf/DELE-", "Asha"]) == \
        "XLa9ec7652b512345933b806fea3bbb2cf/DELE-Asha"


def test_a_line_cut_after_a_dot_inside_an_id_continues_even_when_short():
    assert join_description(["UPI/DEMOFOOD LTD/demofoodltd32.", "rz/PayviaDemo"], [False, False]) == \
        "UPI/DEMOFOOD LTD/demofoodltd32.rz/PayviaDemo"


def test_a_cell_line_break_followed_by_a_space_cuts_a_word_and_any_other_break_is_a_space():
    from statement_reader.segment.wrap import cell_text

    assert cell_text(" WDL TFR   UPI/DR/1/xyautopa\n y./Indi   5123456789284 AT 05150") == \
        "WDL TFR UPI/DR/1/xyautopay./Indi 5123456789284 AT 05150"
    assert cell_text("NEFT*X*Fakeshi\n p Limi") == "NEFT*X*Fakeship Limi"
    assert cell_text("first line\nsecond line") == "first line second line"
