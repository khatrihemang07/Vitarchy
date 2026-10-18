import pytest

from statement_reader.normalize.references import find_reference
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


@pytest.mark.parametrize("description,reference", [
    ("UPI/DR/512345678986/DEMO GRO/HDFC/xyautopay./India", "512345678986"),
    ("UPI-512345679003-PINE TRE E FUELS IN", "512345679003"),
    ("FAKE PERSON 1 UPI/600000000000/NA/fake0@exb/Example Bank", "600000000000"),
    ("NEFT*KKBK0000958*CMS5123456789131*Fakeship Limit", "CMS5123456789131"),
    ("IMPS/412345678901/x", "412345678901"),
])
def test_a_upi_neft_or_imps_reference_is_found_in_the_description(description, reference):
    assert find_reference(description, V) == reference


@pytest.mark.parametrize("description", ["ATM WDL ATM CASH 7844", "Payment received", "", "UPI/12345/short"])
def test_a_description_without_a_reference_gives_none(description):
    assert find_reference(description, V) is None
