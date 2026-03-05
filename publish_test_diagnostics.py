#!/usr/bin/env python3
"""Publish fake DiagnosticArray messages at 1Hz for testing."""
import time
import rclpy
from rclpy.node import Node
from diagnostic_msgs.msg import DiagnosticArray, DiagnosticStatus, KeyValue
from std_msgs.msg import Header


def main():
    rclpy.init()
    node = Node("test_publisher")
    pub = node.create_publisher(DiagnosticArray, "/diagnostics", 10)

    msg = DiagnosticArray()
    status = DiagnosticStatus()
    status.level = DiagnosticStatus.OK
    status.name = "Motor Driver"
    status.message = "OK"
    status.hardware_id = "mc01"
    status.values = [
        KeyValue(key="temperature", value="42.5"),
        KeyValue(key="voltage", value="24.1"),
    ]
    msg.status = [status]

    node.get_logger().info("Publishing DiagnosticArray at 1Hz...")
    try:
        while rclpy.ok():
            msg.header = Header()
            msg.header.stamp = node.get_clock().now().to_msg()
            pub.publish(msg)
            time.sleep(1.0)
    except KeyboardInterrupt:
        pass
    finally:
        node.destroy_node()
        rclpy.shutdown()


if __name__ == "__main__":
    main()
