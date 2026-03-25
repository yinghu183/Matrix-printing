import unittest

from matrix_printing_logic import calculate_grid_metrics, split_text_paragraphs


class CalculateGridMetricsTests(unittest.TestCase):
    def test_calculates_cell_size_for_valid_grid(self):
        metrics = calculate_grid_metrics((2480, 3508), 20, 28)

        self.assertEqual(metrics["start_x"], "0")
        self.assertEqual(metrics["start_y"], "0")
        self.assertEqual(metrics["cell_width"], "124")
        self.assertEqual(metrics["cell_height"], "125")
        self.assertEqual(metrics["font_size"], "120")

    def test_rejects_zero_or_negative_grid_dimensions(self):
        with self.assertRaisesRegex(ValueError, "正整数"):
            calculate_grid_metrics((1000, 1000), 0, 10)

        with self.assertRaisesRegex(ValueError, "正整数"):
            calculate_grid_metrics((1000, 1000), 10, -1)

    def test_clamps_font_size_for_tiny_cells(self):
        metrics = calculate_grid_metrics((3, 3), 3, 3)

        self.assertEqual(metrics["cell_width"], "1")
        self.assertEqual(metrics["cell_height"], "1")
        self.assertEqual(metrics["font_size"], "1")


class SplitTextParagraphsTests(unittest.TestCase):
    def test_splits_on_single_or_multiple_newlines(self):
        text = "第一段\n第二段\n\n\n第三段"

        self.assertEqual(
            split_text_paragraphs(text),
            ["第一段", "第二段", "第三段"],
        )

    def test_ignores_blank_lines_and_crlf_whitespace(self):
        text = "\r\n  第一段  \r\n\r\n   \r\n第二段\r\n"

        self.assertEqual(split_text_paragraphs(text), ["第一段", "第二段"])


if __name__ == "__main__":
    unittest.main()
