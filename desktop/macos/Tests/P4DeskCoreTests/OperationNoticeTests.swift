import XCTest
@testable import P4DeskCore

final class OperationNoticeTests: XCTestCase {
    func testCompletedSuccessDoesNotReplaceUndismissedFailure() {
        var queue = OperationNoticeQueue()
        let failure = OperationNotice(title: "本机保存失败", message: "设备数据仍有效。")
        let success = OperationNotice(title: "同步完成", message: "设备已提交。")
        queue.enqueue(failure); queue.enqueue(success)
        XCTAssertEqual(queue.current, failure)
        queue.presentNext()
        XCTAssertEqual(queue.current, failure)
        queue.dismissCurrent()
        XCTAssertNil(queue.current)
        queue.presentNext()
        XCTAssertEqual(queue.current, success)
        queue.dismissCurrent(); queue.presentNext()
        XCTAssertNil(queue.current)
    }

    func testSuppressionAppliesToSuccessAndFailure() {
        var queue = OperationNoticeQueue()
        XCTAssertFalse(queue.enqueue(OperationNotice(title: "完成", message: "成功"), suppressed: true))
        XCTAssertFalse(queue.enqueue(OperationNotice(title: "失败", message: "错误"), suppressed: true))
        queue.presentNext()
        XCTAssertNil(queue.current)
    }

    func testRepeatedReportDoesNotCreateSeveralAlerts() {
        var queue = OperationNoticeQueue()
        XCTAssertTrue(queue.enqueue(OperationNotice(title: "断线", message: "传输中断")))
        XCTAssertFalse(queue.enqueue(OperationNotice(title: "断线", message: "传输中断")))
        queue.dismissCurrent(); queue.presentNext()
        XCTAssertNil(queue.current)
    }

    func testQueuedReportsRemainInCompletionOrder() {
        var queue = OperationNoticeQueue()
        let first = OperationNotice(title: "一", message: "1")
        let second = OperationNotice(title: "二", message: "2")
        let third = OperationNotice(title: "三", message: "3")
        queue.enqueue(first); queue.enqueue(second); queue.enqueue(third)
        XCTAssertFalse(queue.enqueue(OperationNotice(title: "二", message: "2")))
        queue.dismissCurrent(); queue.presentNext(); XCTAssertEqual(queue.current, second)
        queue.dismissCurrent(); queue.presentNext(); XCTAssertEqual(queue.current, third)
    }

    func testNewResultDuringDismissalDoesNotJumpAheadOfPendingFailure() {
        var queue = OperationNoticeQueue()
        let first = OperationNotice(title: "完成", message: "1")
        let failure = OperationNotice(title: "失败", message: "2")
        let next = OperationNotice(title: "完成", message: "3")
        queue.enqueue(first); queue.enqueue(failure)
        queue.dismissCurrent()
        queue.enqueue(next)
        XCTAssertEqual(queue.current, failure)
        queue.dismissCurrent(); queue.presentNext()
        XCTAssertEqual(queue.current, next)
    }

    func testCapacityPreservesCurrentAndEightWaitingNotices() {
        var queue = OperationNoticeQueue()
        let current = OperationNotice(title: "当前失败", message: "仍未关闭")
        let waiting = (1...8).map { OperationNotice(title: "等待 \($0)", message: "结果 \($0)") }
        queue.enqueue(current)
        for notice in waiting { XCTAssertTrue(queue.enqueue(notice)) }
        XCTAssertFalse(queue.enqueue(OperationNotice(title: "超出容量", message: "不能覆盖已有通知")))
        XCTAssertFalse(queue.enqueue(OperationNotice(title: current.title, message: current.message)))
        XCTAssertEqual(queue.current, current)

        for notice in waiting {
            queue.dismissCurrent(); queue.presentNext()
            XCTAssertEqual(queue.current, notice)
        }
        queue.dismissCurrent(); queue.presentNext()
        XCTAssertNil(queue.current)
        let later = OperationNotice(title: "新结果", message: "容量释放后可继续接收")
        XCTAssertTrue(queue.enqueue(later))
        XCTAssertEqual(queue.current, later)
    }
}
