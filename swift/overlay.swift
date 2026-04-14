import AppKit

// rust callbacks

@_silgen_name("rust_on_key_pressed")
func rustOnKeyPressed(_ keycode: UInt16)

@_silgen_name("rust_on_overlay_dismissed")
func rustOnOverlayDismissed()


// matches the #[repr(C)] OverlayAppearance in ffi.rs. field order matters
struct OverlayAppearance {
    var backgroundOpacity: Double
    var borderR: Double; var borderG: Double; var borderB: Double; var borderA: Double
    var fillR: Double; var fillG: Double; var fillB: Double; var fillA: Double
    var highlightR: Double; var highlightG: Double; var highlightB: Double; var highlightA: Double
    var textR: Double; var textG: Double; var textB: Double; var textA: Double
    var fontSizeRatio: Double
    var borderWidth: Double
    var cellGap: Double
    var cornerRadius: Double
}

// sensible defaults matching the hardcoded values we used to have
extension OverlayAppearance {
    static let `default` = OverlayAppearance(
        backgroundOpacity: 0.55,
        borderR: 0.5, borderG: 0.5, borderB: 1.0, borderA: 0.4,
        fillR: 0.5, fillG: 0.5, fillB: 1.0, fillA: 0.08,
        highlightR: 0.5, highlightG: 0.5, highlightB: 1.0, highlightA: 0.3,
        textR: 0.5, textG: 0.5, textB: 1.0, textA: 0.9,
        fontSizeRatio: 0.4,
        borderWidth: 1.0,
        cellGap: 8.0,
        cornerRadius: 8.0
    )
}

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
        self.backgroundColor = .clear
        self.hasShadow = true
        self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        self.isFloatingPanel = true
        self.becomesKeyOnlyIfNeeded = false
        self.hidesOnDeactivate = false
        self.alphaValue = 0.0
    }

    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }

    func fadeIn(duration: TimeInterval = 0.15) {
        NSAnimationContext.runAnimationGroup { ctx in
            ctx.duration = duration
            ctx.timingFunction = CAMediaTimingFunction(name: .easeOut)
            self.animator().alphaValue = 1.0
        }
    }

    func fadeOut(duration: TimeInterval = 0.12, completion: (() -> Void)? = nil) {
        NSAnimationContext.runAnimationGroup({ ctx in
            ctx.duration = duration
            ctx.timingFunction = CAMediaTimingFunction(name: .easeIn)
            self.animator().alphaValue = 0.0
        }, completionHandler: {
            self.orderOut(nil)
            completion?()
        })
    }
}

class GridView: NSView {
    let cols: Int
    let rows: Int
    let labels: [[String]]
    let overlayAppearance: OverlayAppearance
    var highlightedCol: Int = -1
    var highlightedRow: Int = -1

    static let defaultLabels: [[String]] = [
        ["Q", "W", "E", "R"],
        ["A", "S", "D", "F"],
        ["Z", "X", "C", "V"],
    ]

    private var cellGap: CGFloat { CGFloat(overlayAppearance.cellGap) }
    private var cornerRadius: CGFloat { CGFloat(overlayAppearance.cornerRadius) }

    init(frame: NSRect, cols: Int, rows: Int, labels: [[String]], overlayAppearance: OverlayAppearance) {
        self.cols = cols
        self.rows = rows
        self.labels = labels
        self.overlayAppearance = overlayAppearance
        super.init(frame: frame)
    }

    required init?(coder: NSCoder) {
        fatalError("nope")
    }

    override var acceptsFirstResponder: Bool { true }

    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)
        let a = overlayAppearance

        let totalGapX = cellGap * CGFloat(cols + 1)
        let totalGapY = cellGap * CGFloat(rows + 1)
        let cellWidth = (bounds.width - totalGapX) / CGFloat(cols)
        let cellHeight = (bounds.height - totalGapY) / CGFloat(rows)

        let borderColor = NSColor(red: a.borderR, green: a.borderG, blue: a.borderB, alpha: a.borderA)
        let fillColor = NSColor(red: a.fillR, green: a.fillG, blue: a.fillB, alpha: a.fillA)
        let highlightColor = NSColor(red: a.highlightR, green: a.highlightG, blue: a.highlightB, alpha: a.highlightA)
        let glowColor = NSColor(red: a.highlightR, green: a.highlightG, blue: a.highlightB, alpha: a.highlightA * 0.5)
        let textColor = NSColor(red: a.textR, green: a.textG, blue: a.textB, alpha: a.textA)

        let fontSize = cellHeight * a.fontSizeRatio
        let font = NSFont.monospacedSystemFont(ofSize: fontSize, weight: .semibold)
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
                let flippedRow = rows - 1 - row
                let x = cellGap + CGFloat(col) * (cellWidth + cellGap)
                let y = cellGap + CGFloat(flippedRow) * (cellHeight + cellGap)
                let cellRect = NSRect(x: x, y: y, width: cellWidth, height: cellHeight)

                let isHighlighted = col == highlightedCol && row == highlightedRow
                let path = NSBezierPath(roundedRect: cellRect, xRadius: cornerRadius, yRadius: cornerRadius)
                let ctx = NSGraphicsContext.current!.cgContext

                // drop shadow for depth -- makes each cell look elevated
                ctx.saveGState()
                ctx.setShadow(offset: CGSize(width: 0, height: -2), blur: 6,
                              color: NSColor.black.withAlphaComponent(0.4).cgColor)

                if isHighlighted {
                    highlightColor.setFill()
                } else {
                    fillColor.setFill()
                }
                path.fill()
                ctx.restoreGState()

                // highlight glow (selected cell gets an extra bloom)
                if isHighlighted {
                    ctx.saveGState()
                    ctx.setShadow(offset: .zero, blur: 14, color: glowColor.cgColor)
                    highlightColor.setFill()
                    path.fill()
                    ctx.restoreGState()
                }

                // top-edge light reflection -- thin bright line simulating light on glass
                let reflectionInset: CGFloat = cornerRadius
                let reflectionRect = NSRect(
                    x: cellRect.minX + reflectionInset,
                    y: cellRect.maxY - 1.5,
                    width: cellRect.width - reflectionInset * 2,
                    height: 1.0
                )
                NSColor.white.withAlphaComponent(0.12).setFill()
                let reflectionPath = NSBezierPath(roundedRect: reflectionRect, xRadius: 0.5, yRadius: 0.5)
                reflectionPath.fill()

                borderColor.setStroke()
                path.lineWidth = isHighlighted ? a.borderWidth * 1.5 : a.borderWidth
                path.stroke()

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

    // set by rust before the overlay is ever shown
    var overlayAppearance: OverlayAppearance = .default
    var gridLabels: [[String]] = GridView.defaultLabels
    var gridCols: Int = 4
    var gridRows: Int = 3

    private init() {}

    func show(x: Double, y: Double, width: Double, height: Double) {
        let frame = NSRect(x: x, y: y, width: width, height: height)
        let overlayPanel = OverlayPanel(frame: frame)

        // frosted glass -- hudWindow is the lightest blur material.
        // alphaValue controls how much of the content shows through
        let blur = NSVisualEffectView(frame: NSRect(origin: .zero, size: frame.size))
        blur.blendingMode = .behindWindow
        blur.material = .hudWindow
        blur.state = .active
        blur.alphaValue = min(max(CGFloat(overlayAppearance.backgroundOpacity), 0.0), 1.0)

        let grid = GridView(
            frame: NSRect(origin: .zero, size: frame.size),
            cols: gridCols,
            rows: gridRows,
            labels: gridLabels,
            overlayAppearance: overlayAppearance
        )

        // stack: blur behind, grid on top
        let container = NSView(frame: NSRect(origin: .zero, size: frame.size))
        container.addSubview(blur)
        container.addSubview(grid)

        overlayPanel.contentView = container
        overlayPanel.makeKeyAndOrderFront(nil)
        overlayPanel.makeFirstResponder(grid)
        overlayPanel.fadeIn()

        self.panel = overlayPanel
        self.gridView = grid
    }

    func hide() {
        guard let panel = panel else { return }
        panel.fadeOut { [weak self] in
            self?.panel = nil
            self?.gridView = nil
        }
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

// menu bar icon. NSStatusItem needs to be retained or it vanishes
class StatusItemController: NSObject {
    static let shared = StatusItemController()

    private var statusItem: NSStatusItem?

    func setup() {
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)

        if let button = item.button {
            if let img = NSImage(systemSymbolName: "square.grid.2x2", accessibilityDescription: "Cartographer") {
                img.isTemplate = true
                button.image = img
            } else {
                button.title = "⊞"
            }
        }

        let menu = NSMenu()

        let prefsItem = NSMenuItem(title: "Preferences...", action: nil, keyEquivalent: ",")
        prefsItem.isEnabled = false // placeholder for now
        menu.addItem(prefsItem)

        let restartItem = NSMenuItem(title: "Restart", action: #selector(restart), keyEquivalent: "r")
        restartItem.target = self
        menu.addItem(restartItem)

        menu.addItem(NSMenuItem.separator())

        let quitItem = NSMenuItem(title: "Quit Cartographer", action: #selector(quit), keyEquivalent: "q")
        quitItem.target = self
        menu.addItem(quitItem)

        item.menu = menu
        self.statusItem = item
    }

    @objc func restart() {
        OverlayController.shared.hide()
        // re-exec the same binary. replaces the process in-place, no zombie
        execv(CommandLine.arguments[0], CommandLine.unsafeArgv)
    }

    @objc func quit() {
        // hide overlay if it's up, then bail
        OverlayController.shared.hide()
        NSApp.terminate(nil)
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

@_cdecl("swift_setup_status_item")
func swiftSetupStatusItem() {
    StatusItemController.shared.setup()
}

@_cdecl("swift_configure_appearance")
func swiftConfigureAppearance(_ ptr: UnsafeRawPointer) {
    OverlayController.shared.overlayAppearance = ptr.load(as: OverlayAppearance.self)
}

@_cdecl("swift_configure_grid_labels")
func swiftConfigureGridLabels(_ labelsPtr: UnsafePointer<CChar>) {
    let raw = String(cString: labelsPtr)
    // format: "Q,W,E,R;A,S,D,F;Z,X,C,V"
    let rows = raw.split(separator: ";").map { row in
        row.split(separator: ",").map(String.init)
    }
    if !rows.isEmpty {
        OverlayController.shared.gridLabels = rows
        OverlayController.shared.gridRows = rows.count
        OverlayController.shared.gridCols = rows[0].count
    }
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
