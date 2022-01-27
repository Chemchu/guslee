import { motion } from "framer-motion";
import Image from "next/image";
import { ScreenSide } from "../../../types";

const cardVariants = {
    offScreenLeft: {
        opacity: 0,
        rotate: -30
    },
    offScreenRight: {
        opacity: 0,
        rotate: 30
    },
    onScreenLeft: {
        opacity: 1,
        rotate: 5,
        transition: {
            type: "spring",
            bounce: 0.4,
            duration: 0.8
        }
    },
    onScreenRight: {
        opacity: 1,
        rotate: -5,
        transition: {
            type: "spring",
            bounce: 0.4,
            duration: 0.8
        }
    }
};

const LateralCard = (props: { className?: string, imgSrc: string, side: ScreenSide }) => {
    return (
        <motion.div
            whileInView={props.side === ScreenSide.LeftSide ? cardVariants.onScreenLeft : cardVariants.onScreenRight}
            initial={props.side === ScreenSide.LeftSide ? cardVariants.offScreenLeft : cardVariants.offScreenRight}
            viewport={{ once: true }}
            className={`h-full w-full relative`}
        >
            <Image className={`${props.className} `} src={props.imgSrc} layout="fill" objectFit="cover" />
        </motion.div>
    );
}

export default LateralCard;