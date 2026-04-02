import AppKit

// rust callbacks

@_silgen_name("rust_on_key_pressed")
func rustOnKeyPressed(_ keycode: UInt16)

@_silgen_name("rust_on_overlay_dismissed")
func rustOnOverlayDismissed()

// NSPanel because NSWindow can't become key without activating the app.
// took so fucking long to figure that out, bc I'm not a swift dev
class OverlayPanel: NSPanel {
    init(frame: NSRect) {
        super.init(
            contentRect: frame,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )

        self.level = .popUpMenu
        self.isOpaque = false
        self.backgroundColor = NSColor.black.withAlphaComponent(0.15)
        self.hasShadow = false
        self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        self.isFloatingPanel = true
        self.becomesKeyOnlyIfNeeded = false
        self.hidesOnDeactivate = false
    }

    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }
}

class GridView: NSView {
    let cols: Int
    let rows: Int
    let labels: [[String]]
    var highlightedCol: Int = -1
    var highlightedRow: Int = -1

    static let defaultLabels: [[String]] = [
        ["Q", "W", "E", "R"],
        ["A", "S", "D", "F"],
        ["Z", "X", "C", "V"],
    ]

    init(frame: NSRect, cols: Int = 4, rows: Int = 3) {
        self.cols = cols
        self.rows = rows
        self.labels = GridView.defaultLabels
        super.init(frame: frame)
    }

    required init?(coder: NSCoder) {
        fatalError("nope")
    }

    override var acceptsFirstResponder: Bool { true }

    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)

        let cellWidth = bounds.width / CGFloat(cols)
        let cellHeight = bounds.height / CGFloat(rows)

        let borderColor = NSColor(red: 0.5, green: 0.5, blue: 1.0, alpha: 0.4)
        let fillColor = NSColor(red: 0.5, green: 0.5, blue: 1.0, alpha: 0.08)
        let highlightColor = NSColor(red: 0.5, green: 0.5, blue: 1.0, alpha: 0.3)
        let textColor = NSColor(red: 0.5, green: 0.5, blue: 1.0, alpha: 0.9)

        let fontSize = cellHeight * 0.4
        let font = NSFont.boldSystemFont(ofSize: fontSize)
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.alignment = .center

        let textAttributes: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: textColor,
            .paragraphStyle: paragraphStyle,
        ]

        for row in 0..<rows {
            for col in 0..<cols {
                // NSView y=0 is bottom, row 0 is top. flip it
                let y = bounds.height - CGFloat(row + 1) * cellHeight
                let cellRect = NSRect(
                    x: CGFloat(col) * cellWidth,
                    y: y,
                    width: cellWidth,
                    height: cellHeight
                )

                if col == highlightedCol && row == highlightedRow {
                    highlightColor.setFill()
                } else {
                    fillColor.setFill()
                }
                cellRect.fill()

                borderColor.setStroke()
                let borderPath = NSBezierPath(rect: cellRect)
                borderPath.lineWidth = 1.0
                borderPath.stroke()

                if row < labels.count && col < labels[row].count {
                    let label = labels[row][col]
                    let textSize = label.size(withAttributes: textAttributes)
                    let textRect = NSRect(
                        x: cellRect.midX - textSize.width / 2,
                        y: cellRect.midY - textSize.height / 2,
                        width: textSize.width,
                        height: textSize.height
                    )
                    label.draw(in: textRect, withAttributes: textAttributes)
                }
            }
        }
    }

    override func keyDown(with event: NSEvent) {
        if event.isARepeat { return }
        rustOnKeyPressed(event.keyCode)
    }

    // catches modifier+key combos that keyDown misses
    override func performKeyEquivalent(with event: NSEvent) -> Bool {
        if event.isARepeat { return true }
        if event.type == .keyDown {
            rustOnKeyPressed(event.keyCode)
        }
        return true
    }
}

class OverlayController {
    static let shared = OverlayController()

    private var panel: OverlayPanel?
    private var gridView: GridView?

    private init() {}

    func show(x: Double, y: Double, width: Double, height: Double) {
        let frame = NSRect(x: x, y: y, width: width, height: height)
        let overlayPanel = OverlayPanel(frame: frame)

        let grid = GridView(frame: NSRect(origin: .zero, size: frame.size))
        overlayPanel.contentView = grid
        overlayPanel.makeKeyAndOrderFront(nil)
        overlayPanel.makeFirstResponder(grid)

        self.panel = overlayPanel
        self.gridView = grid
    }

    func hide() {
        panel?.orderOut(nil)
        panel = nil
        gridView = nil
    }

    func highlightCell(col: Int, row: Int) {
        gridView?.highlightedCol = col
        gridView?.highlightedRow = row
        gridView?.needsDisplay = true
    }

    func clearHighlight() {
        gridView?.highlightedCol = -1
        gridView?.highlightedRow = -1
        gridView?.needsDisplay = true
    }
}

// C-callable bridge (rust calls these)

@_cdecl("swift_show_overlay")
func swiftShowOverlay(_ x: Double, _ y: Double, _ width: Double, _ height: Double) {
    OverlayController.shared.show(x: x, y: y, width: width, height: height)
}

@_cdecl("swift_hide_overlay")
func swiftHideOverlay() {
    OverlayController.shared.hide()
}

@_cdecl("swift_highlight_cell")
func swiftHighlightCell(_ col: Int32, _ row: Int32) {
    OverlayController.shared.highlightCell(col: Int(col), row: Int(row))
}

@_cdecl("swift_clear_highlight")
func swiftClearHighlight() {
    OverlayController.shared.clearHighlight()
}

@_cdecl("swift_get_screen_visible_frame")
func swiftGetScreenVisibleFrame(
    _ x: UnsafeMutablePointer<Double>,
    _ y: UnsafeMutablePointer<Double>,
    _ w: UnsafeMutablePointer<Double>,
    _ h: UnsafeMutablePointer<Double>
) {
    guard let screen = NSScreen.main else {
        x.pointee = 0
        y.pointee = 0
        w.pointee = 1920
        h.pointee = 1080
        return
    }
    let frame = screen.visibleFrame
    x.pointee = frame.origin.x
    y.pointee = frame.origin.y
    w.pointee = frame.size.width
    h.pointee = frame.size.height
}
